#[path = "src/vcpu/context.rs"] mod context;

use context::*;
use std::io::Write;

macro_rules! offset_define_add {
    ($list:expr, $name:expr, $ctx:expr, $reg:expr) => {
        $list.push(OffsetDefine::new(String::from($name), field_offset(&$ctx, &$reg)));
    };
}

macro_rules! hu_csr_add {
    ($list:expr, $name:expr, $num:expr) => {
        $list.push(OffsetDefine::new(String::from($name), $num));
    };
}

fn field_offset(ctx: &VcpuCtx, reg: &u64) -> u64 {
    let ctx_ptr = (ctx as *const VcpuCtx) as u64;
    let reg_ptr = (reg as *const u64) as u64;

    reg_ptr - ctx_ptr
}

struct OffsetDefine {
    name: String,
    offset: u64,
}

impl OffsetDefine {
    pub fn new(name: String, offset: u64) -> Self {
        Self {
            name,
            offset,
        }
    }
}

fn main() {
    println!("cargo:warning=------------build.rs start!-------------");
    let vcpu = VcpuCtx::new();
    let mut offset_define_list: Vec<OffsetDefine> = Vec::new();

    // Add offset define
    // Host GP
    offset_define_add!(offset_define_list, "HOST_GP_X0", vcpu, vcpu.host_ctx.gp_regs.x_reg[0]);
    offset_define_add!(offset_define_list, "HOST_GP_X1", vcpu, vcpu.host_ctx.gp_regs.x_reg[1]);
    offset_define_add!(offset_define_list, "HOST_GP_X2", vcpu, vcpu.host_ctx.gp_regs.x_reg[2]);
    offset_define_add!(offset_define_list, "HOST_GP_X3", vcpu, vcpu.host_ctx.gp_regs.x_reg[3]);
    offset_define_add!(offset_define_list, "HOST_GP_X4", vcpu, vcpu.host_ctx.gp_regs.x_reg[4]);
    offset_define_add!(offset_define_list, "HOST_GP_X5", vcpu, vcpu.host_ctx.gp_regs.x_reg[5]);
    offset_define_add!(offset_define_list, "HOST_GP_X6", vcpu, vcpu.host_ctx.gp_regs.x_reg[6]);
    offset_define_add!(offset_define_list, "HOST_GP_X7", vcpu, vcpu.host_ctx.gp_regs.x_reg[7]);
    offset_define_add!(offset_define_list, "HOST_GP_X8", vcpu, vcpu.host_ctx.gp_regs.x_reg[8]);
    offset_define_add!(offset_define_list, "HOST_GP_X9", vcpu, vcpu.host_ctx.gp_regs.x_reg[9]);
    offset_define_add!(offset_define_list, "HOST_GP_X10", vcpu, vcpu.host_ctx.gp_regs.x_reg[10]);
    offset_define_add!(offset_define_list, "HOST_GP_X11", vcpu, vcpu.host_ctx.gp_regs.x_reg[11]);
    offset_define_add!(offset_define_list, "HOST_GP_X12", vcpu, vcpu.host_ctx.gp_regs.x_reg[12]);
    offset_define_add!(offset_define_list, "HOST_GP_X13", vcpu, vcpu.host_ctx.gp_regs.x_reg[13]);
    offset_define_add!(offset_define_list, "HOST_GP_X14", vcpu, vcpu.host_ctx.gp_regs.x_reg[14]);
    offset_define_add!(offset_define_list, "HOST_GP_X15", vcpu, vcpu.host_ctx.gp_regs.x_reg[15]);
    offset_define_add!(offset_define_list, "HOST_GP_X16", vcpu, vcpu.host_ctx.gp_regs.x_reg[16]);
    offset_define_add!(offset_define_list, "HOST_GP_X17", vcpu, vcpu.host_ctx.gp_regs.x_reg[17]);
    offset_define_add!(offset_define_list, "HOST_GP_X18", vcpu, vcpu.host_ctx.gp_regs.x_reg[18]);
    offset_define_add!(offset_define_list, "HOST_GP_X19", vcpu, vcpu.host_ctx.gp_regs.x_reg[19]);
    offset_define_add!(offset_define_list, "HOST_GP_X20", vcpu, vcpu.host_ctx.gp_regs.x_reg[20]);
    offset_define_add!(offset_define_list, "HOST_GP_X21", vcpu, vcpu.host_ctx.gp_regs.x_reg[21]);
    offset_define_add!(offset_define_list, "HOST_GP_X22", vcpu, vcpu.host_ctx.gp_regs.x_reg[22]);
    offset_define_add!(offset_define_list, "HOST_GP_X23", vcpu, vcpu.host_ctx.gp_regs.x_reg[23]);
    offset_define_add!(offset_define_list, "HOST_GP_X24", vcpu, vcpu.host_ctx.gp_regs.x_reg[24]);
    offset_define_add!(offset_define_list, "HOST_GP_X25", vcpu, vcpu.host_ctx.gp_regs.x_reg[25]);
    offset_define_add!(offset_define_list, "HOST_GP_X26", vcpu, vcpu.host_ctx.gp_regs.x_reg[26]);
    offset_define_add!(offset_define_list, "HOST_GP_X27", vcpu, vcpu.host_ctx.gp_regs.x_reg[27]);
    offset_define_add!(offset_define_list, "HOST_GP_X28", vcpu, vcpu.host_ctx.gp_regs.x_reg[28]);
    offset_define_add!(offset_define_list, "HOST_GP_X29", vcpu, vcpu.host_ctx.gp_regs.x_reg[29]);
    offset_define_add!(offset_define_list, "HOST_GP_X30", vcpu, vcpu.host_ctx.gp_regs.x_reg[30]);
    offset_define_add!(offset_define_list, "HOST_GP_X31", vcpu, vcpu.host_ctx.gp_regs.x_reg[31]);

    // Host HYP
    offset_define_add!(offset_define_list, "HOST_HYP_HUSTATUS", vcpu, vcpu.host_ctx.hyp_regs.hustatus);
    offset_define_add!(offset_define_list, "HOST_HYP_HUEDELEG", vcpu, vcpu.host_ctx.hyp_regs.huedeleg);
    offset_define_add!(offset_define_list, "HOST_HYP_HUIDELEG", vcpu, vcpu.host_ctx.hyp_regs.huideleg);
    offset_define_add!(offset_define_list, "HOST_HYP_HUVIP", vcpu, vcpu.host_ctx.hyp_regs.huvip);
    offset_define_add!(offset_define_list, "HOST_HYP_HUIP", vcpu, vcpu.host_ctx.hyp_regs.huip);
    offset_define_add!(offset_define_list, "HOST_HYP_HUIE", vcpu, vcpu.host_ctx.hyp_regs.huie);
    offset_define_add!(offset_define_list, "HOST_HYP_HUGEIP", vcpu, vcpu.host_ctx.hyp_regs.hugeip);
    offset_define_add!(offset_define_list, "HOST_HYP_HUGEIE", vcpu, vcpu.host_ctx.hyp_regs.hugeie);
    offset_define_add!(offset_define_list, "HOST_HYP_HUCOUNTEREN", vcpu, vcpu.host_ctx.hyp_regs.hucounteren);
    offset_define_add!(offset_define_list, "HOST_HYP_HUTIMEDELTA", vcpu, vcpu.host_ctx.hyp_regs.hutimedelta);
    offset_define_add!(offset_define_list, "HOST_HYP_HUTIMEDELTAH", vcpu, vcpu.host_ctx.hyp_regs.hutimedeltah);
    offset_define_add!(offset_define_list, "HOST_HYP_HUTVAL", vcpu, vcpu.host_ctx.hyp_regs.hutval);
    offset_define_add!(offset_define_list, "HOST_HYP_HUTINST", vcpu, vcpu.host_ctx.hyp_regs.hutinst);
    offset_define_add!(offset_define_list, "HOST_HYP_HUGATP", vcpu, vcpu.host_ctx.hyp_regs.hugatp);
    offset_define_add!(offset_define_list, "HOST_HYP_UTVEC", vcpu, vcpu.host_ctx.hyp_regs.utvec);
    offset_define_add!(offset_define_list, "HOST_HYP_UEPC", vcpu, vcpu.host_ctx.hyp_regs.uepc);
    offset_define_add!(offset_define_list, "HOST_HYP_USCRATCH", vcpu, vcpu.host_ctx.hyp_regs.uscratch);
    offset_define_add!(offset_define_list, "HOST_HYP_UTVAL", vcpu, vcpu.host_ctx.hyp_regs.utval);
    offset_define_add!(offset_define_list, "HOST_HYP_UCAUSE", vcpu, vcpu.host_ctx.hyp_regs.ucause);
    offset_define_add!(offset_define_list, "HOST_HYP_SCOUNTEREN", vcpu, vcpu.host_ctx.hyp_regs.scounteren);

    // Guest GP
    offset_define_add!(offset_define_list, "GUEST_GP_X0", vcpu, vcpu.guest_ctx.gp_regs.x_reg[0]);
    offset_define_add!(offset_define_list, "GUEST_GP_X1", vcpu, vcpu.guest_ctx.gp_regs.x_reg[1]);
    offset_define_add!(offset_define_list, "GUEST_GP_X2", vcpu, vcpu.guest_ctx.gp_regs.x_reg[2]);
    offset_define_add!(offset_define_list, "GUEST_GP_X3", vcpu, vcpu.guest_ctx.gp_regs.x_reg[3]);
    offset_define_add!(offset_define_list, "GUEST_GP_X4", vcpu, vcpu.guest_ctx.gp_regs.x_reg[4]);
    offset_define_add!(offset_define_list, "GUEST_GP_X5", vcpu, vcpu.guest_ctx.gp_regs.x_reg[5]);
    offset_define_add!(offset_define_list, "GUEST_GP_X6", vcpu, vcpu.guest_ctx.gp_regs.x_reg[6]);
    offset_define_add!(offset_define_list, "GUEST_GP_X7", vcpu, vcpu.guest_ctx.gp_regs.x_reg[7]);
    offset_define_add!(offset_define_list, "GUEST_GP_X8", vcpu, vcpu.guest_ctx.gp_regs.x_reg[8]);
    offset_define_add!(offset_define_list, "GUEST_GP_X9", vcpu, vcpu.guest_ctx.gp_regs.x_reg[9]);
    offset_define_add!(offset_define_list, "GUEST_GP_X10", vcpu, vcpu.guest_ctx.gp_regs.x_reg[10]);
    offset_define_add!(offset_define_list, "GUEST_GP_X11", vcpu, vcpu.guest_ctx.gp_regs.x_reg[11]);
    offset_define_add!(offset_define_list, "GUEST_GP_X12", vcpu, vcpu.guest_ctx.gp_regs.x_reg[12]);
    offset_define_add!(offset_define_list, "GUEST_GP_X13", vcpu, vcpu.guest_ctx.gp_regs.x_reg[13]);
    offset_define_add!(offset_define_list, "GUEST_GP_X14", vcpu, vcpu.guest_ctx.gp_regs.x_reg[14]);
    offset_define_add!(offset_define_list, "GUEST_GP_X15", vcpu, vcpu.guest_ctx.gp_regs.x_reg[15]);
    offset_define_add!(offset_define_list, "GUEST_GP_X16", vcpu, vcpu.guest_ctx.gp_regs.x_reg[16]);
    offset_define_add!(offset_define_list, "GUEST_GP_X17", vcpu, vcpu.guest_ctx.gp_regs.x_reg[17]);
    offset_define_add!(offset_define_list, "GUEST_GP_X18", vcpu, vcpu.guest_ctx.gp_regs.x_reg[18]);
    offset_define_add!(offset_define_list, "GUEST_GP_X19", vcpu, vcpu.guest_ctx.gp_regs.x_reg[19]);
    offset_define_add!(offset_define_list, "GUEST_GP_X20", vcpu, vcpu.guest_ctx.gp_regs.x_reg[20]);
    offset_define_add!(offset_define_list, "GUEST_GP_X21", vcpu, vcpu.guest_ctx.gp_regs.x_reg[21]);
    offset_define_add!(offset_define_list, "GUEST_GP_X22", vcpu, vcpu.guest_ctx.gp_regs.x_reg[22]);
    offset_define_add!(offset_define_list, "GUEST_GP_X23", vcpu, vcpu.guest_ctx.gp_regs.x_reg[23]);
    offset_define_add!(offset_define_list, "GUEST_GP_X24", vcpu, vcpu.guest_ctx.gp_regs.x_reg[24]);
    offset_define_add!(offset_define_list, "GUEST_GP_X25", vcpu, vcpu.guest_ctx.gp_regs.x_reg[25]);
    offset_define_add!(offset_define_list, "GUEST_GP_X26", vcpu, vcpu.guest_ctx.gp_regs.x_reg[26]);
    offset_define_add!(offset_define_list, "GUEST_GP_X27", vcpu, vcpu.guest_ctx.gp_regs.x_reg[27]);
    offset_define_add!(offset_define_list, "GUEST_GP_X28", vcpu, vcpu.guest_ctx.gp_regs.x_reg[28]);
    offset_define_add!(offset_define_list, "GUEST_GP_X29", vcpu, vcpu.guest_ctx.gp_regs.x_reg[29]);
    offset_define_add!(offset_define_list, "GUEST_GP_X30", vcpu, vcpu.guest_ctx.gp_regs.x_reg[30]);
    offset_define_add!(offset_define_list, "GUEST_GP_X31", vcpu, vcpu.guest_ctx.gp_regs.x_reg[31]);

    //Guest SYS
    offset_define_add!(offset_define_list, "GUEST_SYS_VSSTATUS", vcpu, vcpu.guest_ctx.sys_regs.vsstatus);
    offset_define_add!(offset_define_list, "GUEST_SYS_VSIP", vcpu, vcpu.guest_ctx.sys_regs.vsip);
    offset_define_add!(offset_define_list, "GUEST_SYS_VSIE", vcpu, vcpu.guest_ctx.sys_regs.vsie);
    offset_define_add!(offset_define_list, "GUEST_SYS_VSTEC", vcpu, vcpu.guest_ctx.sys_regs.vstec);
    offset_define_add!(offset_define_list, "GUEST_SYS_VSSCRATCH", vcpu, vcpu.guest_ctx.sys_regs.vsscratch);
    offset_define_add!(offset_define_list, "GUEST_SYS_VSEPC", vcpu, vcpu.guest_ctx.sys_regs.vsepc);
    offset_define_add!(offset_define_list, "GUEST_SYS_VSCAUSE", vcpu, vcpu.guest_ctx.sys_regs.vscause);
    offset_define_add!(offset_define_list, "GUEST_SYS_VSTVAL", vcpu, vcpu.guest_ctx.sys_regs.vstval);
    offset_define_add!(offset_define_list, "GUEST_SYS_VSATP", vcpu, vcpu.guest_ctx.sys_regs.vsatp);
    offset_define_add!(offset_define_list, "GUEST_SYS_VSCOUNTEREN", vcpu, vcpu.guest_ctx.sys_regs.vscounteren);

    // Guest HYP
    offset_define_add!(offset_define_list, "GUEST_HYP_HUSTATUS", vcpu, vcpu.guest_ctx.hyp_regs.hustatus);
    offset_define_add!(offset_define_list, "GUEST_HYP_HUEDELEG", vcpu, vcpu.guest_ctx.hyp_regs.huedeleg);
    offset_define_add!(offset_define_list, "GUEST_HYP_HUIDELEG", vcpu, vcpu.guest_ctx.hyp_regs.huideleg);
    offset_define_add!(offset_define_list, "GUEST_HYP_HUVIP", vcpu, vcpu.guest_ctx.hyp_regs.huvip);
    offset_define_add!(offset_define_list, "GUEST_HYP_HUIP", vcpu, vcpu.guest_ctx.hyp_regs.huip);
    offset_define_add!(offset_define_list, "GUEST_HYP_HUIE", vcpu, vcpu.guest_ctx.hyp_regs.huie);
    offset_define_add!(offset_define_list, "GUEST_HYP_HUGEIP", vcpu, vcpu.guest_ctx.hyp_regs.hugeip);
    offset_define_add!(offset_define_list, "GUEST_HYP_HUGEIE", vcpu, vcpu.guest_ctx.hyp_regs.hugeie);
    offset_define_add!(offset_define_list, "GUEST_HYP_HUCOUNTEREN", vcpu, vcpu.guest_ctx.hyp_regs.hucounteren);
    offset_define_add!(offset_define_list, "GUEST_HYP_HUTIMEDELTA", vcpu, vcpu.guest_ctx.hyp_regs.hutimedelta);
    offset_define_add!(offset_define_list, "GUEST_HYP_HUTIMEDELTAH", vcpu, vcpu.guest_ctx.hyp_regs.hutimedeltah);
    offset_define_add!(offset_define_list, "GUEST_HYP_HUTVAL", vcpu, vcpu.guest_ctx.hyp_regs.hutval);
    offset_define_add!(offset_define_list, "GUEST_HYP_HUTINST", vcpu, vcpu.guest_ctx.hyp_regs.hutinst);
    offset_define_add!(offset_define_list, "GUEST_HYP_HUGATP", vcpu, vcpu.guest_ctx.hyp_regs.hugatp);
    offset_define_add!(offset_define_list, "GUEST_HYP_UTVEC", vcpu, vcpu.guest_ctx.hyp_regs.utvec);
    offset_define_add!(offset_define_list, "GUEST_HYP_UEPC", vcpu, vcpu.guest_ctx.hyp_regs.uepc);
    offset_define_add!(offset_define_list, "GUEST_HYP_USCRATCH", vcpu, vcpu.guest_ctx.hyp_regs.uscratch);
    offset_define_add!(offset_define_list, "GUEST_HYP_UTVAL", vcpu, vcpu.guest_ctx.hyp_regs.utval);
    offset_define_add!(offset_define_list, "GUEST_HYP_UCAUSE", vcpu, vcpu.guest_ctx.hyp_regs.ucause);
    offset_define_add!(offset_define_list, "GUEST_HYP_SCOUNTEREN", vcpu, vcpu.guest_ctx.hyp_regs.scounteren);

    // Write C header file: src/asm-offset.h
    let mut header_file = std::fs::File::create("src/asm-offset.h").expect("create failed");
    header_file.write_all("#ifndef _ASM_REGDEF_H\n#define _ASM_REGDEF_H\n".as_bytes()).expect("write failed");
    header_file.write_all("/* This file is generated by build.rs. Please do not modify it! */\n\n".as_bytes()).expect("write failed");

    for i in &offset_define_list {
        header_file.write_all("#define ".as_bytes()).expect("write failed");
        header_file.write_all(i.name.as_bytes()).expect("write failed");
        header_file.write_all(" ".as_bytes()).expect("write failed");
        header_file.write_all(i.offset.to_string().as_bytes()).expect("write failed");
        header_file.write_all("\n".as_bytes()).expect("write failed");
    }
    header_file.write_all("\n#endif".as_bytes()).expect("write failed");

    // Write rust file: src/vcpu/asmoffset.rs
    let mut rust_file = std::fs::File::create("src/vcpu/asmoffset.rs").expect("create failed");
    rust_file.write_all("/* This file is generated by build.rs. Please do not modify it! */\n\n".as_bytes()).expect("write failed");
    rust_file.write_all("#![allow(unused)]\n".as_bytes()).expect("write failed");

    for i in &offset_define_list {
        rust_file.write_all("\n//".as_bytes()).expect("write failed");
        rust_file.write_all(i.name.as_bytes()).expect("write failed");
        rust_file.write_all("\n".as_bytes()).expect("write failed");
        rust_file.write_all("pub const ".as_bytes()).expect("write failed");
        rust_file.write_all(i.name.as_bytes()).expect("write failed");
        rust_file.write_all(": u64 = ".as_bytes()).expect("write failed");
        rust_file.write_all(i.offset.to_string().as_bytes()).expect("write failed");
        rust_file.write_all(";\n".as_bytes()).expect("write failed");
    }

    // Write asm file: src/asm_offset.S
    let mut asm_file = std::fs::File::create("src/asm_offset.S").expect("create failed");
    asm_file.write_all("/* This file is generated by build.rs. Please do not modify it! */\n\n".as_bytes()).expect("write failed");

    for i in &offset_define_list {
        asm_file.write_all("// ".as_bytes()).expect("write failed");
        asm_file.write_all(i.name.as_bytes()).expect("write failed");
        asm_file.write_all("\n".as_bytes()).expect("write failed");
        asm_file.write_all(".macro SAVE_".as_bytes()).expect("write failed");
        asm_file.write_all(i.name.as_bytes()).expect("write failed");
        asm_file.write_all(" ctx, reg\n".as_bytes()).expect("write failed");
        asm_file.write_all("    sd \\reg, (".as_bytes()).expect("write failed");
        asm_file.write_all(i.offset.to_string().as_bytes()).expect("write failed");
        asm_file.write_all(")(\\ctx)\n.endm\n".as_bytes()).expect("write failed");

        asm_file.write_all(".macro RESTORE_".as_bytes()).expect("write failed");
        asm_file.write_all(i.name.as_bytes()).expect("write failed");
        asm_file.write_all(" ctx, reg\n".as_bytes()).expect("write failed");
        asm_file.write_all("    ld \\reg, (".as_bytes()).expect("write failed");
        asm_file.write_all(i.offset.to_string().as_bytes()).expect("write failed");
        asm_file.write_all(")(\\ctx)\n.endm\n".as_bytes()).expect("write failed");

        asm_file.write_all("\n".as_bytes()).expect("write failed");
    }

    // CSR define
    let mut hu_csr_list: Vec<OffsetDefine> = Vec::new();

    hu_csr_add!(hu_csr_list, "CSR_HUSTATUS", 0x800);
    hu_csr_add!(hu_csr_list, "CSR_HUEDELEG", 0x802);
    hu_csr_add!(hu_csr_list, "CSR_HUIDELEG", 0x803);
    hu_csr_add!(hu_csr_list, "CSR_HUIE", 0x804);
    hu_csr_add!(hu_csr_list, "CSR_HUCOUNTEREN", 0x806);
    hu_csr_add!(hu_csr_list, "CSR_HUTVAL", 0x843);
    hu_csr_add!(hu_csr_list, "CSR_HUVIP", 0x845);
    hu_csr_add!(hu_csr_list, "CSR_HUIP", 0x844);
    hu_csr_add!(hu_csr_list, "CSR_HUTINST", 0x84A);
    hu_csr_add!(hu_csr_list, "CSR_HUGATP", 0x880);
    hu_csr_add!(hu_csr_list, "CSR_HUTIMEDELTA", 0x805);
    hu_csr_add!(hu_csr_list, "CSR_HUTIMEDELTAH", 0x815);
    hu_csr_add!(hu_csr_list, "CSR_UEPC", 0x41);
    hu_csr_add!(hu_csr_list, "CSR_UTVEC", 0x5);
    hu_csr_add!(hu_csr_list, "CSR_USCRATCH", 0x40);


    // Write asm file: src/asm_csr.S
    let mut csr_asm_file = std::fs::File::create("src/asm_csr.S").expect("create failed");
    csr_asm_file.write_all("/* This file is generated by build.rs. Please do not modify it! */\n\n".as_bytes()).expect("write failed");

    for i in &hu_csr_list {

        csr_asm_file.write_all("// ".as_bytes()).expect("write failed");
        csr_asm_file.write_all(i.name.as_bytes()).expect("write failed");
        csr_asm_file.write_all("\n".as_bytes()).expect("write failed");
        csr_asm_file.write_all(".macro CSRRW_".as_bytes()).expect("write failed");
        csr_asm_file.write_all(i.name.as_bytes()).expect("write failed");
        csr_asm_file.write_all(" reg1, reg2\n".as_bytes()).expect("write failed");
        csr_asm_file.write_all("    csrrw \\reg1, ".as_bytes()).expect("write failed");
        csr_asm_file.write_all(i.offset.to_string().as_bytes()).expect("write failed");
        csr_asm_file.write_all(", \\reg2\n.endm\n".as_bytes()).expect("write failed");
        csr_asm_file.write_all("\n".as_bytes()).expect("write failed");

        csr_asm_file.write_all(".macro CSRW_".as_bytes()).expect("write failed");
        csr_asm_file.write_all(i.name.as_bytes()).expect("write failed");
        csr_asm_file.write_all(" reg1\n".as_bytes()).expect("write failed");
        csr_asm_file.write_all("    csrw ".as_bytes()).expect("write failed");
        csr_asm_file.write_all(i.offset.to_string().as_bytes()).expect("write failed");
        csr_asm_file.write_all(", \\reg1\n.endm\n".as_bytes()).expect("write failed");
        csr_asm_file.write_all("\n".as_bytes()).expect("write failed");

        csr_asm_file.write_all(".macro CSRR_".as_bytes()).expect("write failed");
        csr_asm_file.write_all(i.name.as_bytes()).expect("write failed");
        csr_asm_file.write_all(" reg1\n".as_bytes()).expect("write failed");
        csr_asm_file.write_all("    csrr \\reg1, ".as_bytes()).expect("write failed");
        csr_asm_file.write_all(i.offset.to_string().as_bytes()).expect("write failed");
        csr_asm_file.write_all("\n.endm\n".as_bytes()).expect("write failed");
        csr_asm_file.write_all("\n".as_bytes()).expect("write failed");
    }

    println!("cargo:warning=------------build.rs end!---------------");
}