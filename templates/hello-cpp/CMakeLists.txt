cmake_minimum_required(VERSION 3.10)
project(hello-cpp LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_executable(hello-cpp src/main.cpp)

# CMAKE_BUILD_TYPE に応じてコンパイルオプションを設定
if(CMAKE_BUILD_TYPE MATCHES "Debug")
    # デバッグビルドの場合
    message(STATUS "Configuring for Debug build...")
    target_compile_options(hello-cpp PRIVATE
        -g                    # デバッグシンボルを生成 (GCC/Clang)
        $<$<CXX_COMPILER_ID:MSVC>:/Zi> # デバッグ情報を有効にする (MSVC)
        -DDEBUG_MODE          # DEBUG_MODE マクロを定義
    )
    # リンクオプション (デバッグ版のライブラリなど)
    # target_link_options(hello-cpp PRIVATE -lDebugLib)

elseif(CMAKE_BUILD_TYPE MATCHES "Release")
    # リリースビルドの場合
    message(STATUS "Configuring for Release build...")
    target_compile_options(hello-cpp PRIVATE
        -O3                   # 最適化レベル3 (GCC/Clang)
        -DNDEBUG              # assertマクロなどを無効化 (GCC/Clang)
        $<$<CXX_COMPILER_ID:MSVC>:/O2 /DNDEBUG> # 最適化、NDEBUG (MSVC)
    )
    # リンクオプション (リリース版のライブラリなど)
    # target_link_options(hello-cpp PRIVATE -lReleaseLib)

elseif(CMAKE_BUILD_TYPE MATCHES "RelWithDebInfo")
    # デバッグ情報付きリリースビルドの場合
    message(STATUS "Configuring for RelWithDebInfo build...")
    target_compile_options(hello-cpp PRIVATE
        -O2                   # 最適化レベル2
        -g                    # デバッグシンボルを生成
        $<$<CXX_COMPILER_ID:MSVC>:/O2 /Zi> # 最適化、デバッグ情報 (MSVC)
    )

elseif(CMAKE_BUILD_TYPE MATCHES "MinSizeRel")
    # 最小サイズリリースビルドの場合
    message(STATUS "Configuring for MinSizeRel build...")
    target_compile_options(hello-cpp PRIVATE
        -Os                   # サイズ最適化
        $<$<CXX_COMPILER_ID:MSVC>:/Os> # サイズ最適化 (MSVC)
    )
endif()
