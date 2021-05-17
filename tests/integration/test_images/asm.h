/*
 * Copyright (c) 2020 Institute of Parallel And Distributed Systems (IPADS), Shanghai Jiao Tong University (SJTU)
 * ChCore is licensed under the Mulan PSL v1.
 * You can use this software according to the terms and conditions of the Mulan PSL v1.
 * You may obtain a copy of Mulan PSL v1 at:
 *   http://license.coscl.org.cn/MulanPSL
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR
 *   IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR
 *   PURPOSE.
 *   See the Mulan PSL v1 for more details.
 */

#pragma once

#define ECALL_VM_TEST_END (0xFF)

// Opensbi EXT ID
#define SBI_EXT_0_1_SET_TIMER (0x0)
#define SBI_EXT_0_1_CONSOLE_PUTCHAR (0x1)
#define SBI_EXT_0_1_CONSOLE_GETCHAR (0x2)
#define SBI_EXT_0_1_CLEAR_IPI (0x3)
#define SBI_EXT_0_1_SEND_IPI (0x4)
#define SBI_EXT_0_1_REMOTE_FENCE_I (0x5)
#define SBI_EXT_0_1_REMOTE_SFENCE_VMA (0x6)
#define SBI_EXT_0_1_REMOTE_SFENCE_VMA_ASID (0x7)
#define SBI_EXT_0_1_SHUTDOWN (0x8)

#define BEGIN_FUNC(_name)        \
	.global _name;           \
	.type _name, % function; \
	_name:

#define END_FUNC(_name) .size _name, .- _name

#define __FILE_NAME_NAME_END(filename) filename ## _ ## end
#define _FILE_NAME_END(filename)   __FILE_NAME_NAME_END(filename)
#define FILE_NAME_END _FILE_NAME_END( __FILENAME__ )


#define BEGIN_FUNC_FILE_NAME()        \
	.global __FILENAME__;           \
	.type __FILENAME__, % function; \
	.align 12;          			\
	__FILENAME__:


#define END_FUNC_FILE_NAME() 		\
	.global FILE_NAME_END;			\
	FILE_NAME_END:					\
	.size __FILENAME__, .- __FILENAME__
