package com.flowspeed.lib.downloader.utils

import okio.Buffer
import okio.FileSystem
import okio.HashingSource
import okio.Path.Companion.toOkioPath
import okio.Source
import java.io.File

enum class ChecksumAlgorithm {
    MD5,
    SHA1,
    SHA256,
}

object ChecksumUtil {

    private const val BUFFER_SIZE = 64 * 1024L

    fun calculateChecksum(source: Source, algorithm: ChecksumAlgorithm): String {
        val hashingSource = when (algorithm) {
            ChecksumAlgorithm.MD5 -> HashingSource.md5(source)
            ChecksumAlgorithm.SHA1 -> HashingSource.sha1(source)
            ChecksumAlgorithm.SHA256 -> HashingSource.sha256(source)
        }

        val sink = Buffer()
        hashingSource.use { hSource ->
            while (hSource.read(sink, BUFFER_SIZE) != -1L) {
                sink.clear()
            }
        }

        return hashingSource.hash.hex()
    }

    fun calculateChecksum(file: File, algorithm: ChecksumAlgorithm): String {
        if (!file.exists() || !file.isFile) {
            throw IllegalArgumentException("Target file does not exist or is not a regular file: ${file.absolutePath}")
        }

        return FileSystem.SYSTEM.source(file.toOkioPath()).use { source ->
            calculateChecksum(source, algorithm)
        }
    }

    fun detectAlgorithm(hash: String): ChecksumAlgorithm? {
        val clean = hash.trim()
        return when (clean.length) {
            32 -> ChecksumAlgorithm.MD5
            40 -> ChecksumAlgorithm.SHA1
            64 -> ChecksumAlgorithm.SHA256
            else -> null
        }
    }

    fun verifyFileAuto(file: File, expectedHash: String): Boolean {
        val algorithm = detectAlgorithm(expectedHash) ?: return false
        return verifyChecksum(file, expectedHash, algorithm)
    }

    fun verifyChecksum(file: File, expectedHash: String, algorithm: ChecksumAlgorithm): Boolean {
        val calculated = calculateChecksum(file, algorithm)
        return calculated.equals(expectedHash.trim(), ignoreCase = true)
    }

    fun verifyChecksum(source: Source, expectedHash: String, algorithm: ChecksumAlgorithm): Boolean {
        val calculated = calculateChecksum(source, algorithm)
        return calculated.equals(expectedHash.trim(), ignoreCase = true)
    }
}
