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

BEGIN_FUNC(vm_test_mem_6)
	li		t0,	0x3000
	ld		t1,	0(t0)
	addi	t2,	t1,	1
	sd		t2,	0(t0)
	jalr	t0
	li		t0,	1
	ecall

.global vm_test_mem_6_end;
vm_test_mem_6_end:
END_FUNC(vm_test_mem_6)
