package com.flowspeed.lib.downloader.part

import com.flowspeed.lib.downloader.annotation.HeavyCall
import com.flowspeed.lib.downloader.connection.Connection
import com.flowspeed.lib.downloader.connection.IResponseInfo
import com.flowspeed.lib.downloader.destination.DestWriter
import com.flowspeed.lib.downloader.exception.DownloadValidationException
import com.flowspeed.lib.downloader.exception.PartTooManyErrorException
import com.flowspeed.lib.downloader.utils.ExceptionUtils
import com.flowspeed.lib.downloader.utils.printStackIfNotUsual
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.isActive
import kotlinx.coroutines.job
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import okio.Buffer
import okio.Source
import okio.use
import kotlin.math.min

const val PART_MAX_TRIES = 10
const val RetryDelay = 1_000L
const val OPTIMAL_PART_BUFFER_SIZE = 64 * 1024L

abstract class PartDownloader<
        TPart : DownloadPart
        >(
    val part: TPart,
    val getDestWriter: () -> DestWriter
) {
    private var downloadJob: Job? = null
    private var scope: CoroutineScope? = null
    private val _statusFlow = part.statusFlow
    val statusFlow = _statusFlow.asStateFlow()

    @Volatile
    internal var active = false

    abstract fun howMuchCanRead(maxAllowed: Long): Long

    @Volatile
    internal var tries = 0

    // make sure to not lake resource in this exception
    @Volatile
    private var lastCriticalException: Throwable? = null

    // make sure to not lake resource in this exception
    @Volatile
    private var lastException: Throwable? = null

    //just turn on (fast)
    fun start() {
        synchronized(this) {
            if (active) {
                return
            }
            stop = false
            active = true
        }
        val scope = CoroutineScope(SupervisorJob()).also {
            this.scope = it
        }
        scope.launch {
            tries = 0
            lastCriticalException = null
            lastException = null
            val result = runCatching {
                while (coroutineContext.isActive || !stop) {
                    if (tries > 0) {
                        delay(RetryDelay)
                    }
                    if (haveToManyErrors()) {
                        iCantRetryAnymore(
                            PartTooManyErrorException(
                                part,
                                lastException
                                    ?: Exception("BUG : if you see me please report it to the developer! when we encounter error so it have to be a least one last exception"),
                            )
                        )
                    }
                    if (part.isCompleted) {
                        onFinish()
                        break
                    }
                    try {
                        download()
                    } catch (e: Exception) {
                        tries++
                        onCanceled(e)
                        when (canRetry(e)) {
                            CanRetryResult.Yes -> continue
                            CanRetryResult.No -> {}
                            CanRetryResult.NoAndStopDownloadJob -> iCantRetryAnymore(e)
                        }
                        break
                    }
                    //download progress started, but maybe we have errors
                    //wait for a finish/error event...
                    //await for cancel status to be emitted!
                    val status = withContext(NonCancellable) {
                        awaitFinishOrError()
                    }
                    when (status) {
                        is PartDownloadStatus.Canceled -> {
                            tries++
                            when (canRetry(status.e)) {
                                CanRetryResult.Yes -> continue
                                CanRetryResult.No -> {}
                                CanRetryResult.NoAndStopDownloadJob -> iCantRetryAnymore(status.e)
                            }
                            break
                        }

                        PartDownloadStatus.Completed -> break
                        else -> throw ShouldNotHappened("should not happened!")
                    }
                }
            }

            active = false
            if (!part.isCompleted) {
                part.statusFlow.value = PartDownloadStatus.IDLE
            }

            result.onFailure {
                if (it is ShouldNotHappened) {
                    throw it
                }
            }
        }
    }

    @Volatile
    var stop = false
    fun stop() {
        stop = true
        downloadJob?.cancel()
        scope?.coroutineContext?.job?.cancel()
    }

    suspend fun join() {
        withContext(Dispatchers.IO) {
            scope?.coroutineContext?.job?.join()
            downloadJob?.join()
        }
    }


    private fun canRetry(e: Throwable): CanRetryResult {
        return when {
            ExceptionUtils.isNormalCancellation(e) -> {
                CanRetryResult.No
            }

            e is DownloadValidationException -> if (e.isCritical()) {
                //download validation occurs, and also it is critical,
                //so we can't proceed any further
                CanRetryResult.NoAndStopDownloadJob
            } else {
                CanRetryResult.Yes
            }

            else -> {
                CanRetryResult.Yes
            }
        }
    }

    lateinit var onTooManyErrors: ((Throwable) -> Unit)
    private fun iCantRetryAnymore(throwable: Throwable) {
        lastCriticalException = throwable
        val currentScope = scope
        if (currentScope?.isActive == true) {
            currentScope.launch {
                onTooManyErrors(throwable)
            }
        } else {
            onTooManyErrors(throwable)
        }
    }

    private fun haveToManyErrors(): Boolean {
        return tries >= PART_MAX_TRIES
    }

    private fun haveCriticalError(): Boolean {
        return lastCriticalException != null
    }

    internal fun injured(): Boolean {
        return haveToManyErrors() || haveCriticalError()
    }

    abstract suspend fun connectAndVerify(): Connection<*>

    private suspend fun download() {
        onNewStatus(PartDownloadStatus.Connecting)
        val conn = connectAndVerify()
        val currentScope = scope ?: return
        downloadJob = currentScope.launch(Dispatchers.IO) {
            if (stop || !isActive) {
                conn.close()
                onCanceled(kotlinx.coroutines.CancellationException())
                return@launch
            }
//            thisLogger().info("going to copy data to destination $conn")
            try {
                conn.use {
                    // connection automatically closes the source
                    val connectionStream = it.source
                    getDestWriter().use { writer ->
                        copyDataSync(connectionStream, writer)
                    }
                }
            } catch (e: Exception) {
                onCanceled(e)
            } finally {
                downloadJob = null
            }
        }
    }

    protected open fun onCanceled(e: Throwable) {
        lastException = e
        val canceled = PartDownloadStatus.Canceled(e)
        onNewStatus(canceled)
        e.printStackIfNotUsual()
    }

    protected open fun onFinish() {
        onNewStatus(PartDownloadStatus.Completed)
    }

    fun onNewStatus(partDownloadStatus: PartDownloadStatus) {
        _statusFlow.value = partDownloadStatus
    }

    @HeavyCall
    private fun copyDataSync(source: Source, destWriter: DestWriter) {
        val buffer = Buffer()
        var totalReadCount = 0L
        var firstLoop = true
        val bufferSize = OPTIMAL_PART_BUFFER_SIZE
        while (true) {
            if (stop) {
                onCanceled(kotlinx.coroutines.CancellationException())
                break
            }
            val howMuchICanReadAllowed = howMuchCanRead(bufferSize)
            val homMuchReadFromBuffer = min(bufferSize, howMuchICanReadAllowed)
//            require(part.current + homMuchReadFromBuffer <= part.maxAllowedCurrent) {
//                """$partSplitSupport
//                canRead:${homMuchReadFromBuffer}"""
//            }
//            require(part.current + homMuchReadFromBuffer <= partSplitSupport.safeZone + 1) {
//                """a
//                    part=${part} isCompleted =${part.isCompleted}
//                    split part $partSplitSupport
//                    howMuch:${homMuchReadFromBuffer}
//                    actual:${part.current + homMuchReadFromBuffer}
//                    expected:${partSplitSupport.safeZone}
//                """.trimIndent()
//            }
            if (howMuchICanReadAllowed <= 0) {
                if (part.isCompleted) {
                    onFinish()
                } else {
                    onCanceled(kotlinx.coroutines.CancellationException("it seems our part was split so we are canceled $part"))
                }
                break
            }
            val readCount = source.read(buffer, homMuchReadFromBuffer)
//            require(readCount <= homMuchReadFromBuffer) {
//                "read count $readCount is bigger than homMuchReadFromBuffer $homMuchReadFromBuffer"
//            }
            if (readCount == -1L) {
                onFinish()
                break
            }
            destWriter.write(buffer, readCount)
            totalReadCount += readCount
            part.current += readCount
//                require (part.current-part.from == totalReadCount)
            if (firstLoop) {
                tries = 0
                onNewStatus(PartDownloadStatus.ReceivingData)
                firstLoop = false
            }
        }
    }

    suspend fun awaitFinishOrError(): PartDownloadStatus {
        return statusFlow.filter {
            when (it) {
                PartDownloadStatus.Completed,
                is PartDownloadStatus.Canceled,
                    -> true

                PartDownloadStatus.ReceivingData,
                PartDownloadStatus.Connecting,
                PartDownloadStatus.IDLE,
                    -> false
            }
        }.first()
    }

    suspend fun awaitToEnsureDataBeingTransferred(): Boolean {
        return withTimeoutOrNull(5_000) {
            val isThatOk = statusFlow.filter {
                when (it) {
                    PartDownloadStatus.Completed,
                    PartDownloadStatus.ReceivingData,
                        -> true

                    is PartDownloadStatus.Canceled,
                    PartDownloadStatus.Connecting,
                    PartDownloadStatus.IDLE,
                        -> false
                }
            }.first().let {
                when (it) {
                    is PartDownloadStatus.Canceled -> false
                    PartDownloadStatus.Completed -> true
                    PartDownloadStatus.ReceivingData -> true
                    PartDownloadStatus.Connecting,
                    PartDownloadStatus.IDLE,
                        -> error("should not happen")
                }
            }
            isThatOk
        } ?: false
    }

    suspend fun awaitIdle() {
        statusFlow.filter {
            when (it) {
                is PartDownloadStatus.Canceled,
                PartDownloadStatus.Completed,
                PartDownloadStatus.IDLE,
                    -> true

                PartDownloadStatus.Connecting,
                PartDownloadStatus.ReceivingData,
                    -> false
            }
        }.first()
    }

    class ShouldNotHappened(msg: String?) : RuntimeException(msg)
    private sealed interface CanRetryResult {
        data object Yes : CanRetryResult
        data object No : CanRetryResult
        data object NoAndStopDownloadJob : CanRetryResult
    }
}


