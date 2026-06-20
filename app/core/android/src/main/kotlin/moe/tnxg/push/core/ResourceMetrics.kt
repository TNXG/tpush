package moe.tnxg.push.core

import android.app.ActivityManager
import android.content.Context

object ResourceMetrics {
    fun snapshot(context: Context): Snapshot {
        val activityManager = context.getSystemService(Context.ACTIVITY_SERVICE) as ActivityManager
        val packageName = context.packageName
        val runningProcesses = activityManager.runningAppProcesses.orEmpty()
        val frontendProcess = runningProcesses.firstOrNull { process -> process.processName == packageName }
        val serviceProcess = runningProcesses.firstOrNull { process -> process.processName == "$packageName:tpush" }

        val frontendMemory = frontendProcess?.let { memoryForPid(activityManager, it.pid) }
        val serviceMemory = serviceProcess?.let { memoryForPid(activityManager, it.pid) }

        return Snapshot(
            serviceRunning = serviceProcess != null,
            servicePid = serviceProcess?.pid ?: 0,
            frontendPid = frontendProcess?.pid ?: 0,
            corePssKb = serviceMemory?.totalPssKb ?: 0,
            frontendPssKb = frontendMemory?.totalPssKb ?: 0,
            coreNativeHeapKb = serviceMemory?.nativeHeapAllocatedKb ?: 0,
            frontendNativeHeapKb = frontendMemory?.nativeHeapAllocatedKb ?: 0,
        )
    }

    private fun memoryForPid(activityManager: ActivityManager, pid: Int): ProcessMemory {
        val memoryInfo = activityManager.getProcessMemoryInfo(intArrayOf(pid)).first()
        return ProcessMemory(
            totalPssKb = memoryInfo.totalPss,
            nativeHeapAllocatedKb = memoryInfo.nativePss,
        )
    }

    data class Snapshot(
        val serviceRunning: Boolean,
        val servicePid: Int,
        val frontendPid: Int,
        val corePssKb: Int,
        val frontendPssKb: Int,
        val coreNativeHeapKb: Int,
        val frontendNativeHeapKb: Int,
    )

    private data class ProcessMemory(
        val totalPssKb: Int,
        val nativeHeapAllocatedKb: Int,
    )
}
