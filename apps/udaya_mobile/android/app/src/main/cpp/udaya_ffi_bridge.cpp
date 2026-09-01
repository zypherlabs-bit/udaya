#include <jni.h>
#include <string>
#include <android/log.h>

#define LOG_TAG "udaya_ffi_mobile"
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

extern "C" {

JNIEXPORT jstring JNICALL
Java_com_udaya_mobile_UdayaNativeBridge_getNodeStatusJson(JNIEnv *env, jobject thiz, jlong nodeHandle) {
    LOGI("getNodeStatusJson called with handle: %lld", (long long) nodeHandle);
    return env->NewStringUTF("{\"status\":\"running\",\"height\":0}");
}

JNIEXPORT jstring JNICALL
Java_com_udaya_mobile_UdayaNativeBridge_getNetworkInfoJson(JNIEnv *env, jobject thiz, jlong nodeHandle) {
    LOGI("getNetworkInfoJson called");
    return env->NewStringUTF("{\"network\":\"mainnet\",\"peerCount\":0}");
}

JNIEXPORT jint JNICALL
Java_com_udaya_mobile_UdayaNativeBridge_nodeStart(JNIEnv *env, jobject thiz, jstring configPath) {
    LOGI("nodeStart called");
    return 0;
}

JNIEXPORT jint JNICALL
Java_com_udaya_mobile_UdayaNativeBridge_nodeStop(JNIEnv *env, jobject thiz, jlong nodeHandle) {
    LOGI("nodeStop called");
    return 0;
}

}
