package com.flowspeed.lib.downloader.utils

import okio.Buffer
import java.io.File
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class ChecksumUtilTest {

    @Test
    fun `calculateChecksum on source returns correct MD5, SHA1 and SHA256 hashes`() {
        val sampleData = "Hello, FlowSpeed Download Manager!"

        val md5Source = Buffer().writeUtf8(sampleData)
        val md5 = ChecksumUtil.calculateChecksum(md5Source, ChecksumAlgorithm.MD5)
        assertEquals("fea974a904d6c5937a7697dac8580bc0", md5)

        val sha1Source = Buffer().writeUtf8(sampleData)
        val sha1 = ChecksumUtil.calculateChecksum(sha1Source, ChecksumAlgorithm.SHA1)
        assertEquals("4feb0a5b6e32801115700d8c4eb1b5b019974c7b", sha1)

        val sha256Source = Buffer().writeUtf8(sampleData)
        val sha256 = ChecksumUtil.calculateChecksum(sha256Source, ChecksumAlgorithm.SHA256)
        assertEquals("74436f21063fd774a8d29e5ba05eece9833e65cf89f9e811769a7161fb4ca829", sha256)
    }

    @Test
    fun `verifyChecksum verifies matching and non-matching hashes case-insensitively`() {
        val sampleData = "Hello, FlowSpeed Download Manager!"
        val source = Buffer().writeUtf8(sampleData)

        // Uppercase expected hash should match
        assertTrue(
            ChecksumUtil.verifyChecksum(
                source = source,
                expectedHash = "FEA974A904D6C5937A7697DAC8580BC0",
                algorithm = ChecksumAlgorithm.MD5
            )
        )

        val sourceMismatch = Buffer().writeUtf8(sampleData)
        assertFalse(
            ChecksumUtil.verifyChecksum(
                source = sourceMismatch,
                expectedHash = "00000000000000000000000000000000",
                algorithm = ChecksumAlgorithm.MD5
            )
        )
    }

    @Test
    fun `calculateChecksum on File matches source checksum`() {
        val tempFile = File.createTempFile("flowspeed_checksum_test", ".tmp")
        try {
            tempFile.writeText("FlowSpeed Core Downloader High-Performance Stream")

            val expectedSha256 = ChecksumUtil.calculateChecksum(
                Buffer().writeUtf8("FlowSpeed Core Downloader High-Performance Stream"),
                ChecksumAlgorithm.SHA256
            )

            val fileSha256 = ChecksumUtil.calculateChecksum(tempFile, ChecksumAlgorithm.SHA256)
            assertEquals(expectedSha256, fileSha256)
            assertTrue(ChecksumUtil.verifyChecksum(tempFile, expectedSha256, ChecksumAlgorithm.SHA256))
            assertTrue(ChecksumUtil.verifyFileAuto(tempFile, expectedSha256))
        } finally {
            tempFile.delete()
        }
    }

    @Test
    fun `detectAlgorithm recognizes MD5, SHA1 and SHA256 length correctly`() {
        assertEquals(ChecksumAlgorithm.MD5, ChecksumUtil.detectAlgorithm("fea974a904d6c5937a7697dac8580bc0"))
        assertEquals(ChecksumAlgorithm.SHA1, ChecksumUtil.detectAlgorithm("4feb0a5b6e32801115700d8c4eb1b5b019974c7b"))
        assertEquals(ChecksumAlgorithm.SHA256, ChecksumUtil.detectAlgorithm("74436f21063fd774a8d29e5ba05eece9833e65cf89f9e811769a7161fb4ca829"))
        assertEquals(null, ChecksumUtil.detectAlgorithm("invalid_length_hash"))
    }
}
