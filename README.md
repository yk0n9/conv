# Conv

使用FFmpeg合并音频，图片和字幕生成视频的工具

支持Whisper语音识别

### 使用
下载[FFmpeg](https://github.com/BtbN/FFmpeg-Builds/releases/latest)并设置环境变量(ffmpeg/bin)

注意：图片尺寸不要过大，保持在小于1MB，高度小于1080px

Scoop:
```
scoop install ffmpeg
```

运行conv.exe

### 构建
安装
[CMake](https://cmake.org/download/)

[Rustup](https://rustup.rs/)

[LLVM](https://releases.llvm.org/download.html)

[C++ 构建工具](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

Scoop:
```
scoop install cmake rustup llvm
```
