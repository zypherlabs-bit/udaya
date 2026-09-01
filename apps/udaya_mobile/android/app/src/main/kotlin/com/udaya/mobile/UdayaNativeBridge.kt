package com.udaya.mobile

object UdayaNativeBridge {
    external fun nodeStart(configPath: String?): Int
    external fun nodeStop(nodeHandle: Long): Int
    external fun getNodeStatusJson(nodeHandle: Long): String?
    external fun getNetworkInfoJson(nodeHandle: Long): String?
    external fun walletCreate(network: String?): Long
    external fun walletDestroy(walletHandle: Long)
    external fun walletGenerateAddress(walletHandle: Long): String?
    external fun walletGetBalance(walletHandle: Long): Long
    external fun walletSend(walletHandle: Long, address: String?, amountSats: Long, feeRate: Long, resultJson: String?): Int
    external fun blockchainGetHeight(nodeHandle: Long): Int
    external fun blockchainGetBlockHash(nodeHandle: Long, height: Int): String?
    external fun getLastError(): String?

    init {
        System.loadLibrary("udaya_ffi_bridge")
    }
}
