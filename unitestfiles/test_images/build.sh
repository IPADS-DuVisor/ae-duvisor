#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

main() {
	SCRIPT_DIR=`dirname "$0"`
	BUILD_DIR_TMP=$1
	BUILD_DIR=${BUILD_DIR_TMP:-build}
	TARGET_DIR=$2
	CLEAN_FLAG=$3
	if [ $CLEAN_FLAG == clean ]; then
		echo "clean all test images"
		rm -rf $BUILD_DIR
		rm ./$TARGET_DIR/*.img
		exit 0
	fi
	rm -rf $BUILD_DIR && mkdir -p $BUILD_DIR

	cd $BUILD_DIR
	echo "compiling vm test images..."

	cmake \
		-DCMAKE_LINKER=riscv64-linux-gnu-ld \
		-DCMAKE_C_LINK_EXECUTABLE="<CMAKE_LINKER> <LINK_FLAGS> <OBJECTS> -o <TARGET> <LINK_LIBRARIES>" \
		-DCMAKE_ASM_LINK_EXECUTABLE="<CMAKE_LINKER> <LINK_FLAGS> <OBJECTS> -o <TARGET> <LINK_LIBRARIES>" \
		${SCRIPT_DIR} -G Ninja ..

	ninja
	cd -
	echo "vm test images compile succeed!"
	mv ./$BUILD_DIR/*.img ./$TARGET_DIR/
	exit 0
}

main $@
