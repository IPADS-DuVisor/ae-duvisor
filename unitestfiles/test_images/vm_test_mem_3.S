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

BEGIN_FUNC(vm_test_mem_3)
	li	t1,	0x3000
	ld	t1,	0(t1)
	li	t2,	0x3000
	li	t3,	0x4321
	sd	t3,	0(t2)
	li	t0, 0x1
	ecall

.global vm_test_mem_3_end;
vm_test_mem_3_end:
END_FUNC(vm_test_mem_3)
