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

#include "asm.h"

BEGIN_FUNC(vm_test_mem_7)
	li		t0,	0
	li		t1,	1024
	li		t2,	0x3000
	li		t4, 0

continue:
	ld		t3,	0(t2)
	add		t4, t4, t3
	li		t6, 0xff8
	add		t2, t2, t6
	ld		t3,	0(t2)
	add	t4,	t4,	t3
	addi	t0,	t0, 1
	li		t5,	0x8
	add		t2,	t2,	t5
	bne		t0,	t1, continue

	li		t0, 1
	ecall

.global vm_test_mem_7_end;
vm_test_mem_7_end:
END_FUNC(vm_test_mem_7)
