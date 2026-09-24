package com.flowspeed.lib.downloader.exception

class ChecksumMismatchException(
    val expected: String,
    val actual: String,
    val algorithm: String,
) : DownloadValidationException(
    "Downloaded file integrity check failed for $algorithm. Expected: $expected, Actual: $actual"
) {
    override fun isCritical(): Boolean = true
}
