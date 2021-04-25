#[path = "src/vcpu/vcpucontext.rs"] mod vcpucontext;

use vcpucontext::*;
use std::io::Write;

macro_rules! offset_define_add {
    ($list:expr, $name:expr, $ctx:expr, $reg:expr) => {
        $list.push(ContextOffset::new(String::from($name), field_offset(&$ctx, &$reg)));
    };
}

fn field_offset<T1, T2>(ctx: &T1, reg: &T2) -> u64 {
    let ctx_ptr = (ctx as *const T1) as u64;
    let reg_ptr = (reg as *const T2) as u64;

    reg_ptr - ctx_ptr
}

struct ContextOffset {
    name: String,
    offset: u64,
}

impl ContextOffset {
    pub fn new(name: String, offset: u64) -> Self {
        Self {
            name,
            offset,
        }
    }
}

fn create_gp_list() -> Vec<ContextOffset> {
    let gp = GpRegs::new();
    let mut gp_list: Vec<ContextOffset> = Vec::new();

    offset_define_add!(gp_list, "X0", gp, gp.x_reg[0]);
    offset_define_add!(gp_list, "X1", gp, gp.x_reg[1]);
    offset_define_add!(gp_list, "X2", gp, gp.x_reg[2]);
    offset_define_add!(gp_list, "X3", gp, gp.x_reg[3]);
    offset_define_add!(gp_list, "X4", gp, gp.x_reg[4]);
    offset_define_add!(gp_list, "X5", gp, gp.x_reg[5]);
    offset_define_add!(gp_list, "X6", gp, gp.x_reg[6]);
    offset_define_add!(gp_list, "X7", gp, gp.x_reg[7]);
    offset_define_add!(gp_list, "X8", gp, gp.x_reg[8]);
    offset_define_add!(gp_list, "X9", gp, gp.x_reg[9]);
    offset_define_add!(gp_list, "X10", gp, gp.x_reg[10]);
    offset_define_add!(gp_list, "X11", gp, gp.x_reg[11]);
    offset_define_add!(gp_list, "X12", gp, gp.x_reg[12]);
    offset_define_add!(gp_list, "X13", gp, gp.x_reg[13]);
    offset_define_add!(gp_list, "X14", gp, gp.x_reg[14]);
    offset_define_add!(gp_list, "X15", gp, gp.x_reg[15]);
    offset_define_add!(gp_list, "X16", gp, gp.x_reg[16]);
    offset_define_add!(gp_list, "X17", gp, gp.x_reg[17]);
    offset_define_add!(gp_list, "X18", gp, gp.x_reg[18]);
    offset_define_add!(gp_list, "X19", gp, gp.x_reg[19]);
    offset_define_add!(gp_list, "X20", gp, gp.x_reg[20]);
    offset_define_add!(gp_list, "X21", gp, gp.x_reg[21]);
    offset_define_add!(gp_list, "X22", gp, gp.x_reg[22]);
    offset_define_add!(gp_list, "X23", gp, gp.x_reg[23]);
    offset_define_add!(gp_list, "X24", gp, gp.x_reg[24]);
    offset_define_add!(gp_list, "X25", gp, gp.x_reg[25]);
    offset_define_add!(gp_list, "X26", gp, gp.x_reg[26]);
    offset_define_add!(gp_list, "X27", gp, gp.x_reg[27]);
    offset_define_add!(gp_list, "X28", gp, gp.x_reg[28]);
    offset_define_add!(gp_list, "X29", gp, gp.x_reg[29]);
    offset_define_add!(gp_list, "X30", gp, gp.x_reg[30]);
    offset_define_add!(gp_list, "X31", gp, gp.x_reg[31]);

    gp_list
}

fn create_sys_list() -> Vec<ContextOffset> {
    let sys = SysRegs::new();
    let mut sys_list: Vec<ContextOffset> = Vec::new();

    offset_define_add!(sys_list, "HUVSSTATUS", sys, sys.huvsstatus);
    offset_define_add!(sys_list, "HUVSIP", sys, sys.huvsip);
    offset_define_add!(sys_list, "HUVSIE", sys, sys.huvsie);
    offset_define_add!(sys_list, "HUVSTVEC", sys, sys.huvstvec);
    offset_define_add!(sys_list, "HUVSSCRATCH", sys, sys.huvsscratch);
    offset_define_add!(sys_list, "HUVSEPC", sys, sys.huvsepc);
    offset_define_add!(sys_list, "HUVSCAUSE", sys, sys.huvscause);
    offset_define_add!(sys_list, "HUVSTVAL", sys, sys.huvstval);
    offset_define_add!(sys_list, "HUVSATP", sys, sys.huvsatp);

    sys_list
}

fn create_hyp_list() -> Vec<ContextOffset> {
    let hyp = HypRegs::new();
    let mut hyp_list: Vec<ContextOffset> = Vec::new();

    offset_define_add!(hyp_list, "HUSTATUS", hyp, hyp.hustatus);
    offset_define_add!(hyp_list, "HUEDELEG", hyp, hyp.huedeleg);
    offset_define_add!(hyp_list, "HUIDELEG", hyp, hyp.huideleg);
    offset_define_add!(hyp_list, "HUVIP", hyp, hyp.huvip);
    offset_define_add!(hyp_list, "HUIP", hyp, hyp.huip);
    offset_define_add!(hyp_list, "HUIE", hyp, hyp.huie);
    offset_define_add!(hyp_list, "HUGEIP", hyp, hyp.hugeip);
    offset_define_add!(hyp_list, "HUGEIE", hyp, hyp.hugeie);
    offset_define_add!(hyp_list, "HUCOUNTEREN", hyp, hyp.hucounteren);
    offset_define_add!(hyp_list, "HUTIMEDELTA", hyp, hyp.hutimedelta);
    offset_define_add!(hyp_list, "HUTIMEDELTAH", hyp, hyp.hutimedeltah);
    offset_define_add!(hyp_list, "HUTVAL", hyp, hyp.hutval);
    offset_define_add!(hyp_list, "HUTINST", hyp, hyp.hutinst);
    offset_define_add!(hyp_list, "HUGATP", hyp, hyp.hugatp);
    offset_define_add!(hyp_list, "UTVEC", hyp, hyp.utvec);
    offset_define_add!(hyp_list, "UEPC", hyp, hyp.uepc);
    offset_define_add!(hyp_list, "USCRATCH", hyp, hyp.uscratch);
    offset_define_add!(hyp_list, "UTVAL", hyp, hyp.utval);
    offset_define_add!(hyp_list, "UCAUSE", hyp, hyp.ucause);

    hyp_list
}

// VcpuCtx.####Ctx.TYPE - VcpuCtx.####Ctx.TYPE.reg
fn create_type_offset(mut offset_define_list: Vec<ContextOffset>, 
    gp_list: &Vec<ContextOffset>, sys_list: &Vec<ContextOffset>,
    hyp_list: &Vec<ContextOffset>) -> Vec<ContextOffset>{
    for i in gp_list {
        let mut full_name = "GP_".to_string();
        let reg_name = i.name.to_string();
        full_name += &reg_name;

        offset_define_list.push(ContextOffset::new(full_name, i.offset));
    }

    for i in sys_list {
        let mut full_name = "SYS_".to_string();
        let reg_name = i.name.to_string();
        full_name += &reg_name;

        offset_define_list.push(ContextOffset::new(full_name, i.offset));
    }

    for i in hyp_list {
        let mut full_name = "HYP_".to_string();
        let reg_name = i.name.to_string();
        full_name += &reg_name;

        offset_define_list.push(ContextOffset::new(full_name, i.offset));
    }

    offset_define_list
}

// VcpuCtx - VcpuCtx.##Ctx.##Regs.reg
fn create_ctx_offset(mut offset_define_list: Vec<ContextOffset>, 
    gp_list: &Vec<ContextOffset>, sys_list: &Vec<ContextOffset>,
    hyp_list: &Vec<ContextOffset>) -> Vec<ContextOffset>{
    let vcpu = VcpuCtx::new();

    // HOST_GP
    for i in gp_list {
        let mut full_name = "HOST_GP_".to_string();
        let reg_name = i.name.to_string();
        full_name += &reg_name;

        let mut offset = field_offset(&vcpu, &vcpu.host_ctx.gp_regs) + i.offset;
        offset_define_list.push(ContextOffset::new(full_name, offset));

        full_name = "GUEST_GP_".to_string();
        full_name += &reg_name;

        offset = field_offset(&vcpu, &vcpu.guest_ctx.gp_regs) + i.offset;
        offset_define_list.push(ContextOffset::new(full_name, offset));
    }

    // HOST_HYP & GUEST_HYP
    for i in hyp_list {
        let mut full_name = "HOST_HYP_".to_string();
        let reg_name = i.name.to_string();
        full_name += &reg_name;

        let mut offset = field_offset(&vcpu, &vcpu.host_ctx.hyp_regs) + i.offset;
        offset_define_list.push(ContextOffset::new(full_name, offset));

        full_name = "GUEST_HYP_".to_string();
        full_name += &reg_name;

        offset = field_offset(&vcpu, &vcpu.guest_ctx.hyp_regs) + i.offset;
        offset_define_list.push(ContextOffset::new(full_name, offset));
    }

    // GUEST_SYS
    for i in sys_list {
        let mut full_name = "GUEST_SYS_".to_string();
        let reg_name = i.name.to_string();
        full_name += &reg_name;

        let offset = field_offset(&vcpu, &vcpu.guest_ctx.sys_regs) + i.offset;
        offset_define_list.push(ContextOffset::new(full_name, offset));
    }

    offset_define_list
}

// VcpuCtx - VcpuCtx.####Ctx.GpRegs
fn create_gp_offset(mut offset_define_list: Vec<ContextOffset>) -> Vec<ContextOffset>{
    let vcpu = VcpuCtx::new();
    
    offset_define_add!(offset_define_list, "HOST_GP", vcpu, vcpu.host_ctx.gp_regs);
    offset_define_add!(offset_define_list, "GUEST_GP", vcpu, vcpu.guest_ctx.gp_regs);

    offset_define_list
}

fn write_asm_offset_header(offset_define_list: Vec<ContextOffset>) {
    let mut asm_offset = std::fs::File::create("guestentry/asm_offset.h").expect("create failed");
    asm_offset.write_all("/* This file is generated by build.rs. Please do not modify it! */\n\n".as_bytes()).expect("write failed");

    for i in offset_define_list {
        asm_offset.write_all("#define ".as_bytes()).expect("write failed");
        asm_offset.write_all(i.name.as_bytes()).expect("write failed");
        asm_offset.write_all(" ".as_bytes()).expect("write failed");
        asm_offset.write_all(i.offset.to_string().as_bytes()).expect("write failed");
        asm_offset.write_all("\n".as_bytes()).expect("write failed");
    }
}

pub fn prepare_asm_offset_header() {
    let mut offset_define_list: Vec<ContextOffset> = Vec::new();
    let gp_list = create_gp_list();
    let sys_list = create_sys_list();
    let hyp_list = create_hyp_list();

    offset_define_list = create_type_offset(offset_define_list, &gp_list, 
        &sys_list, &hyp_list);
    offset_define_list = create_ctx_offset(offset_define_list, &gp_list, 
        &sys_list, &hyp_list);
    offset_define_list = create_gp_offset(offset_define_list);

    write_asm_offset_header(offset_define_list);
}
