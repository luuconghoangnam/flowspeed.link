package com.flowspeed.lib.downloader.annotation

/**
 * annotate that a method has a long-running operation and
 * should not be used in the main thread
 */
@Retention(AnnotationRetention.SOURCE)
annotation class HeavyCall
