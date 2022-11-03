use super::reserved_idt_entries::*;
use super::{Cpu, CpuFlag, RegisterId, Size};
use crate::debug_println;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

pub type InstructionResult = Result<(), u8>;

fn get_sign_bit(value: u64, size: Size) -> bool {
    (value >> ((size as u64) * 8 - 1) & 1) > 0
}

fn trunucate_value(value: u64, size: Size) -> u64 {
    match size {
        Size::One => value as u8 as u64,
        Size::Two => value as u16 as u64,
        Size::Four => value as u32 as u64,
        Size::Eight => value,
    }
}

// Returns true on signed addition overflow
fn does_signed_add_overflow(lhs: u64, rhs: u64, size: Size) -> bool {
    match size {
        Size::One => (lhs as i8).checked_add(rhs as i8).is_none(),
        Size::Two => (lhs as i16).checked_add(rhs as i16).is_none(),
        Size::Four => (lhs as i32).checked_add(rhs as i32).is_none(),
        Size::Eight => (lhs as i64).checked_add(rhs as i64).is_none(),
    }
}

// Returns true on signed subtraction overflow
fn does_signed_sub_overflow(lhs: u64, rhs: u64, size: Size) -> bool {
    match size {
        Size::One => (lhs as i8).checked_sub(rhs as i8).is_none(),
        Size::Two => (lhs as i16).checked_sub(rhs as i16).is_none(),
        Size::Four => (lhs as i32).checked_sub(rhs as i32).is_none(),
        Size::Eight => (lhs as i64).checked_sub(rhs as i64).is_none(),
    }
}

// Returns true on signed mul overflow
fn does_signed_mul_overflow(lhs: u64, rhs: u64, size: Size) -> bool {
    match size {
        Size::One => (lhs as i8).checked_mul(rhs as i8).is_none(),
        Size::Two => (lhs as i16).checked_mul(rhs as i16).is_none(),
        Size::Four => (lhs as i32).checked_mul(rhs as i32).is_none(),
        Size::Eight => (lhs as i64).checked_mul(rhs as i64).is_none(),
    }
}

// Returns true on unsigned mul overflow
fn does_unsigned_mul_overflow(lhs: u64, rhs: u64, size: Size) -> bool {
    match size {
        Size::One => (lhs as u8).checked_mul(rhs as u8).is_none(),
        Size::Two => (lhs as u16).checked_mul(rhs as u16).is_none(),
        Size::Four => (lhs as u32).checked_mul(rhs as u32).is_none(),
        Size::Eight => lhs.checked_mul(rhs).is_none(),
    }
}

// Returns true on signed division overflow
fn does_signed_div_overflow(lhs: u64, rhs: u64, size: Size) -> bool {
    match size {
        Size::One => (lhs as i8).checked_div(rhs as i8).is_none(),
        Size::Two => (lhs as i16).checked_div(rhs as i16).is_none(),
        Size::Four => (lhs as i32).checked_div(rhs as i32).is_none(),
        Size::Eight => (lhs as i64).checked_div(rhs as i64).is_none(),
    }
}

// Returns true on unsigned division overflow
fn does_unsigned_div_overflow(lhs: u64, rhs: u64, size: Size) -> bool {
    match size {
        Size::One => (lhs as u8).checked_div(rhs as u8).is_none(),
        Size::Two => (lhs as u16).checked_div(rhs as u16).is_none(),
        Size::Four => (lhs as u32).checked_div(rhs as u32).is_none(),
        Size::Eight => lhs.checked_div(rhs).is_none(),
    }
}

fn get_effective_address(cpu: &mut Cpu) -> u64 {
    let fetched_byte = cpu.fetch_byte();

    let base_id: Option<RegisterId> = RegisterId::from_u8(fetched_byte & 0b111);
    let index_id: Option<RegisterId> = RegisterId::from_u8(fetched_byte >> 3 & 0b111);

    let const_offset = cpu.fetch_qword();

    let base_value = if let Some(base_id) = base_id {
        cpu.register(base_id)
    } else {
        0
    };

    let index_value = if let Some(index_id) = index_id {
        cpu.register(index_id)
    } else {
        0
    };

    let address = base_value + index_value + const_offset;

    debug_println!("Parsed address: {:#x}", address);

    address
}

impl Cpu {
    pub(super) fn HLT(&mut self) -> InstructionResult {
        debug_println!("X0:       {} ({0:#x})", self.register(RegisterId::X0));
        debug_println!("X1:       {} ({0:#x})", self.register(RegisterId::X1));
        debug_println!("X2:       {} ({0:#x})", self.register(RegisterId::X2));
        debug_println!("X3:       {} ({0:#x})", self.register(RegisterId::X3));
        debug_println!("X4:       {} ({0:#x})", self.register(RegisterId::X4));
        debug_println!("SP:       {} ({0:#x})", self.register(RegisterId::Sp));
        debug_println!("IP:       {} ({0:#x})", self.register(RegisterId::Ip));
        debug_println!("Negative: {}", self.get_flag(CpuFlag::Negative) as u8);
        debug_println!("Zero:     {}", self.get_flag(CpuFlag::Zero) as u8);
        debug_println!("Carry:    {}", self.get_flag(CpuFlag::Carry) as u8);
        debug_println!("Overflow: {}", self.get_flag(CpuFlag::Overflow) as u8);

        self.halted = true;

        Ok(())
    }

    pub(super) fn MOV(&mut self) -> InstructionResult {
        let fetched_byte = self.fetch_byte();

        let src_id: Option<RegisterId> = RegisterId::from_u8(fetched_byte & 0b111);

        let dst_id: RegisterId = match RegisterId::from_u8(fetched_byte >> 3 & 0b111) {
            Some(reg_id) => reg_id,
            None => return Err(INVALID_INSTRUCTION),
        };

        // let dst_idx: RegisterId = RegisterId::try_from((fetched_byte >> 3 & 0b111));
        let size: Size = Size::try_from(1 << (fetched_byte >> 6 & 0b11))
            .expect("Unrecoverable error. Size is not 1, 2, 4, or 8");

        let move_value: u64;

        if let Some(src_id) = src_id {
            move_value = self.register(src_id);
        } else {
            move_value = self.fetch_sized(size);
        }

        debug_println!("Moving {} to {:?} with size {:?}", move_value, dst_id, size);

        self.register_assign_sized(dst_id, move_value, size);

        Ok(())
    }

    pub(super) fn ADD(&mut self) -> InstructionResult {
        let fetched_byte = self.fetch_byte();

        let src_id: Option<RegisterId> = RegisterId::from_u8(fetched_byte & 0b111);

        let dst_id: RegisterId = match RegisterId::from_u8(fetched_byte >> 3 & 0b111) {
            Some(reg_id) => reg_id,
            None => return Err(INVALID_INSTRUCTION),
        };

        // let dst_idx: RegisterId = RegisterId::try_from((fetched_byte >> 3 & 0b111));
        let size: Size = Size::try_from(1 << (fetched_byte >> 6 & 0b11))
            .expect("Unrecoverable error. Size is not 1, 2, 4, or 8");

        let rhs_value: u64;

        if let Some(src_id) = src_id {
            rhs_value = self.register(src_id);
        } else {
            rhs_value = self.fetch_sized(size);
        }

        debug_println!("Adding {:?} with {}", dst_id, rhs_value);

        let result = self.register(dst_id).wrapping_add(rhs_value);

        self.set_flag(CpuFlag::Zero, trunucate_value(result, size) == 0);
        self.set_flag(CpuFlag::Negative, get_sign_bit(result, size));

        self.set_flag(
            CpuFlag::Carry,
            trunucate_value(self.register(dst_id), size) > trunucate_value(result, size),
        );

        self.set_flag(
            CpuFlag::Overflow,
            does_signed_add_overflow(self.register(dst_id), rhs_value, size),
        );

        // self.set_flag(
        //     CpuFlag::Overflow,
        //     get_sign_bit(self.register(dst_id), size) == get_sign_bit(rhs_value, size)
        //         && get_sign_bit(rhs_value, size) != get_sign_bit(result, size),
        // );

        self.register_assign_sized(dst_id, result, size);

        Ok(())
    }

    pub(super) fn SUB(&mut self) -> InstructionResult {
        let fetched_byte = self.fetch_byte();

        let src_id: Option<RegisterId> = RegisterId::from_u8(fetched_byte & 0b111);

        let dst_id: RegisterId = match RegisterId::from_u8(fetched_byte >> 3 & 0b111) {
            Some(reg_id) => reg_id,
            None => return Err(INVALID_INSTRUCTION),
        };

        // let dst_idx: RegisterId = RegisterId::try_from((fetched_byte >> 3 & 0b111));
        let size: Size = Size::try_from(1 << (fetched_byte >> 6 & 0b11))
            .expect("Unrecoverable error. Size is not 1, 2, 4, or 8");

        let rhs_value: u64;

        if let Some(src_id) = src_id {
            rhs_value = self.register(src_id);
        } else {
            rhs_value = self.fetch_sized(size);
        }

        debug_println!("Subtracting {} from {:?}", rhs_value, dst_id);

        let result = self.register(dst_id).wrapping_sub(rhs_value);

        self.set_flag(CpuFlag::Zero, result == 0);
        self.set_flag(CpuFlag::Negative, get_sign_bit(result, size));

        self.set_flag(
            CpuFlag::Carry,
            trunucate_value(self.register(dst_id), size) < trunucate_value(result, size),
        );

        self.set_flag(
            CpuFlag::Overflow,
            does_signed_sub_overflow(self.register(dst_id), rhs_value, size),
        );

        self.register_assign_sized(dst_id, result, size);

        Ok(())
    }

    pub(super) fn MUL(&mut self) -> InstructionResult {
        let fetched_byte = self.fetch_byte();

        let src_id: Option<RegisterId> = RegisterId::from_u8(fetched_byte & 0b111);

        let dst_id: RegisterId = match RegisterId::from_u8(fetched_byte >> 3 & 0b111) {
            Some(reg_id) => reg_id,
            None => return Err(INVALID_INSTRUCTION),
        };

        // let dst_idx: RegisterId = RegisterId::try_from((fetched_byte >> 3 & 0b111));
        let size: Size = Size::try_from(1 << (fetched_byte >> 6 & 0b11))
            .expect("Unrecoverable error. Size is not 1, 2, 4, or 8");

        let rhs_value: u64;

        if let Some(src_id) = src_id {
            rhs_value = self.register(src_id);
        } else {
            rhs_value = self.fetch_sized(size);
        }

        debug_println!("Multiplying {:?} with {}", dst_id, rhs_value);

        let result = self.register(dst_id).wrapping_mul(rhs_value);

        self.set_flag(CpuFlag::Zero, result == 0);
        self.set_flag(CpuFlag::Zero, get_sign_bit(result, size));

        self.set_flag(
            CpuFlag::Carry,
            does_unsigned_mul_overflow(self.register(dst_id), rhs_value, size),
        );
        self.set_flag(
            CpuFlag::Overflow,
            does_signed_mul_overflow(self.register(dst_id), rhs_value, size),
        );

        self.register_assign_sized(dst_id, result, size);

        Ok(())
    }

    pub(super) fn DIV(&mut self) -> InstructionResult {
        let fetched_byte = self.fetch_byte();

        let src_id: Option<RegisterId> = RegisterId::from_u8(fetched_byte & 0b111);

        let dst_id: RegisterId = match RegisterId::from_u8(fetched_byte >> 3 & 0b111) {
            Some(reg_id) => reg_id,
            None => return Err(INVALID_INSTRUCTION),
        };

        // let dst_idx: RegisterId = RegisterId::try_from((fetched_byte >> 3 & 0b111));
        let size: Size = Size::try_from(1 << (fetched_byte >> 6 & 0b11))
            .expect("Unrecoverable error. Size is not 1, 2, 4, or 8");

        let rhs_value: u64;

        if let Some(src_id) = src_id {
            rhs_value = self.register(src_id);
        } else {
            rhs_value = self.fetch_sized(size);
        }

        debug_println!("Dividing {} from {:?}", rhs_value, dst_id);

        if (rhs_value == 0) {
            return Err(DIVIDE_BY_ZERO);
        }

        let result = self.register(dst_id).wrapping_div(rhs_value);

        self.set_flag(CpuFlag::Zero, result == 0);
        self.set_flag(CpuFlag::Zero, get_sign_bit(result, size));

        self.set_flag(
            CpuFlag::Carry,
            does_unsigned_div_overflow(self.register(dst_id), rhs_value, size),
        );

        self.register_assign_sized(dst_id, result, size);

        Ok(())
    }

    pub(super) fn OR(&mut self) -> InstructionResult {
        let fetched_byte = self.fetch_byte();

        let src_id: Option<RegisterId> = RegisterId::from_u8(fetched_byte & 0b111);

        let dst_id: RegisterId = match RegisterId::from_u8(fetched_byte >> 3 & 0b111) {
            Some(reg_id) => reg_id,
            None => return Err(INVALID_INSTRUCTION),
        };

        // let dst_idx: RegisterId = RegisterId::try_from((fetched_byte >> 3 & 0b111));
        let size: Size = Size::try_from(1 << (fetched_byte >> 6 & 0b11))
            .expect("Unrecoverable error. Size is not 1, 2, 4, or 8");

        let rhs_value: u64;

        if let Some(src_id) = src_id {
            rhs_value = self.register(src_id);
        } else {
            rhs_value = self.fetch_sized(size);
        }

        debug_println!("Or-ing {:?} with {}", dst_id, rhs_value);

        let result = self.register(dst_id) | rhs_value;

        self.set_flag(CpuFlag::Zero, result == 0);
        self.set_flag(CpuFlag::Zero, get_sign_bit(result, size));

        self.register_assign_sized(dst_id, result, size);

        Ok(())
    }

    pub(super) fn XOR(&mut self) -> InstructionResult {
        let fetched_byte = self.fetch_byte();

        let src_id: Option<RegisterId> = RegisterId::from_u8(fetched_byte & 0b111);

        let dst_id: RegisterId = match RegisterId::from_u8(fetched_byte >> 3 & 0b111) {
            Some(reg_id) => reg_id,
            None => return Err(INVALID_INSTRUCTION),
        };

        // let dst_idx: RegisterId = RegisterId::try_from((fetched_byte >> 3 & 0b111));
        let size: Size = Size::try_from(1 << (fetched_byte >> 6 & 0b11))
            .expect("Unrecoverable error. Size is not 1, 2, 4, or 8");

        let rhs_value: u64;

        if let Some(src_id) = src_id {
            rhs_value = self.register(src_id);
        } else {
            rhs_value = self.fetch_sized(size);
        }

        debug_println!("Xor-ing {:?} with {}", dst_id, rhs_value);

        let result = self.register(dst_id) ^ rhs_value;

        self.set_flag(CpuFlag::Zero, result == 0);
        self.set_flag(CpuFlag::Zero, get_sign_bit(result, size));

        self.register_assign_sized(dst_id, result, size);

        Ok(())
    }

    pub(super) fn AND(&mut self) -> InstructionResult {
        let fetched_byte = self.fetch_byte();

        let src_id: Option<RegisterId> = RegisterId::from_u8(fetched_byte & 0b111);

        let dst_id: RegisterId = match RegisterId::from_u8(fetched_byte >> 3 & 0b111) {
            Some(reg_id) => reg_id,
            None => return Err(INVALID_INSTRUCTION),
        };

        // let dst_idx: RegisterId = RegisterId::try_from((fetched_byte >> 3 & 0b111));
        let size: Size = Size::try_from(1 << (fetched_byte >> 6 & 0b11))
            .expect("Unrecoverable error. Size is not 1, 2, 4, or 8");

        let rhs_value: u64;

        if let Some(src_id) = src_id {
            rhs_value = self.register(src_id);
        } else {
            rhs_value = self.fetch_sized(size);
        }

        debug_println!("Anding {:?} with {}", dst_id, rhs_value);

        let result = self.register(dst_id) & rhs_value;

        self.set_flag(CpuFlag::Zero, result == 0);
        self.set_flag(CpuFlag::Zero, get_sign_bit(result, size));

        self.register_assign_sized(dst_id, result, size);

        Ok(())
    }

    pub(super) fn NOT(&mut self) -> InstructionResult {
        let fetched_byte = self.fetch_byte();

        let dst_id: RegisterId = match RegisterId::from_u8(fetched_byte & 0b111) {
            Some(reg_id) => reg_id,
            None => return Err(INVALID_INSTRUCTION),
        };

        let size: Size = Size::try_from(1 << (fetched_byte >> 6 & 0b11))
            .expect("Unrecoverable error. Size is not 1, 2, 4, or 8");

        debug_println!("Not-ing {:?}", dst_id);

        let result = !self.register(dst_id);

        self.set_flag(CpuFlag::Zero, result == 0);
        self.set_flag(CpuFlag::Negative, get_sign_bit(result, size));

        self.register_assign_sized(dst_id, result, size);

        Ok(())
    }

    pub(super) fn NEG(&mut self) -> InstructionResult {
        let fetched_byte = self.fetch_byte();

        let dst_id: RegisterId = match RegisterId::from_u8(fetched_byte & 0b111) {
            Some(reg_id) => reg_id,
            None => return Err(INVALID_INSTRUCTION),
        };

        let size: Size = Size::try_from(1 << (fetched_byte >> 6 & 0b11))
            .expect("Unrecoverable error. Size is not 1, 2, 4, or 8");

        debug_println!("Negating {:?}", dst_id);

        let result = self.register(dst_id).wrapping_neg();

        self.set_flag(CpuFlag::Zero, result == 0);
        self.set_flag(CpuFlag::Negative, get_sign_bit(result, size));

        self.register_assign_sized(dst_id, result, size);

        Ok(())
    }

    pub(super) fn CMP(&mut self) -> InstructionResult {
        let fetched_byte = self.fetch_byte();

        let src_id: Option<RegisterId> = RegisterId::from_u8(fetched_byte & 0b111);

        let dst_id: RegisterId = match RegisterId::from_u8(fetched_byte >> 3 & 0b111) {
            Some(reg_id) => reg_id,
            None => return Err(INVALID_INSTRUCTION),
        };

        let size: Size = Size::try_from(1 << (fetched_byte >> 6 & 0b11))
            .expect("Unrecoverable error. Size is not 1, 2, 4, or 8");

        let rhs_value: u64;

        if let Some(src_id) = src_id {
            rhs_value = self.register(src_id);
        } else {
            rhs_value = self.fetch_sized(size);
        }

        debug_println!("Comparing {:?} with {}", dst_id, rhs_value);

        let result = self.register(dst_id).wrapping_sub(rhs_value);

        self.set_flag(CpuFlag::Zero, result == 0);
        self.set_flag(CpuFlag::Negative, get_sign_bit(result, size));

        self.set_flag(
            CpuFlag::Carry,
            trunucate_value(self.register(dst_id), size) < trunucate_value(result, size),
        );

        self.set_flag(
            CpuFlag::Overflow,
            does_signed_sub_overflow(self.register(dst_id), rhs_value, size),
        );

        Ok(())
    }

    pub(super) fn PUSH(&mut self) -> InstructionResult {
        let fetched_byte = self.fetch_byte();

        let src_id: RegisterId = match RegisterId::from_u8(fetched_byte & 0b111) {
            Some(reg_id) => reg_id,
            None => return Err(INVALID_INSTRUCTION),
        };

        debug_println!("Pushing register {:?}", src_id);

        self.push_qword(self.register(src_id));

        Ok(())
    }

    pub(super) fn POP(&mut self) -> InstructionResult {
        let fetched_byte = self.fetch_byte();

        let dst_id: RegisterId = match RegisterId::from_u8(fetched_byte & 0b111) {
            Some(reg_id) => reg_id,
            None => return Err(INVALID_INSTRUCTION),
        };

        debug_println!("Popping stack into register {:?}", dst_id);

        let popped = self.pop_qword();
        self.register_assign(dst_id, popped);

        Ok(())
    }

    pub(super) fn PUSHF(&mut self) -> InstructionResult {
        self.push_flags();

        Ok(())
    }

    pub(super) fn POPF(&mut self) -> InstructionResult {
        self.pop_flags();

        Ok(())
    }

    pub(super) fn STR(&mut self) -> InstructionResult {
        let fetched_byte = self.fetch_byte();

        let src_id: RegisterId = match RegisterId::from_u8(fetched_byte & 0b111) {
            Some(reg_id) => reg_id,
            None => return Err(INVALID_INSTRUCTION),
        };

        let size: Size = Size::try_from(1 << (fetched_byte >> 6 & 0b11))
            .expect("Unrecoverable error. Size is not 1, 2, 4, or 8");

        let address = get_effective_address(self);

        self.write(&self.register(src_id).to_le_bytes(), address);

        Ok(())
    }

    pub(super) fn LDR(&mut self) -> InstructionResult {
        let fetched_byte = self.fetch_byte();

        let dst_id: RegisterId = match RegisterId::from_u8(fetched_byte & 0b111) {
            Some(reg_id) => reg_id,
            None => return Err(INVALID_INSTRUCTION),
        };

        let size: Size = Size::try_from(1 << (fetched_byte >> 6 & 0b11))
            .expect("Unrecoverable error. Size is not 1, 2, 4, or 8");

        let address = get_effective_address(self);

        let mut derefrenced: [u8; 8] = [0; 8];
        self.read(&mut derefrenced, address);

        self.register_assign_sized(dst_id, u64::from_le_bytes(derefrenced), size);

        Ok(())
    }

    pub(super) fn LEA(&mut self) -> InstructionResult {
        let fetched_byte = self.fetch_byte();

        let dst_id: RegisterId = match RegisterId::from_u8(fetched_byte & 0b111) {
            Some(reg_id) => reg_id,
            None => return Err(INVALID_INSTRUCTION),
        };

        let size: Size = Size::try_from(1 << (fetched_byte >> 6 & 0b11))
            .expect("Unrecoverable error. Size is not 1, 2, 4, or 8");

        let address = get_effective_address(self);

        self.register_assign_sized(dst_id, address, size);

        Ok(())
    }

    pub(super) fn JMP(&mut self) -> InstructionResult {
        let address = get_effective_address(self);

        self.register_assign(RegisterId::Ip, address);

        Ok(())
    }

    pub(super) fn JZ(&mut self) -> InstructionResult {
        let address = get_effective_address(self);

        if self.get_flag(CpuFlag::Zero) {
            self.register_assign(RegisterId::Ip, address);
        }

        Ok(())
    }

    pub(super) fn JNZ(&mut self) -> InstructionResult {
        let address = get_effective_address(self);

        if self.get_flag(CpuFlag::Zero) == false {
            self.register_assign(RegisterId::Ip, address);
        }

        Ok(())
    }

    pub(super) fn JO(&mut self) -> InstructionResult {
        let address = get_effective_address(self);

        if self.get_flag(CpuFlag::Overflow) {
            self.register_assign(RegisterId::Ip, address);
        }

        Ok(())
    }

    pub(super) fn JNO(&mut self) -> InstructionResult {
        let address = get_effective_address(self);

        if self.get_flag(CpuFlag::Overflow) == false {
            self.register_assign(RegisterId::Ip, address);
        }

        Ok(())
    }

    pub(super) fn JS(&mut self) -> InstructionResult {
        let address = get_effective_address(self);

        if self.get_flag(CpuFlag::Negative) {
            self.register_assign(RegisterId::Ip, address);
        }

        Ok(())
    }

    pub(super) fn JNS(&mut self) -> InstructionResult {
        let address = get_effective_address(self);

        if self.get_flag(CpuFlag::Negative) == false {
            self.register_assign(RegisterId::Ip, address);
        }

        Ok(())
    }

    pub(super) fn JC(&mut self) -> InstructionResult {
        let address = get_effective_address(self);

        if self.get_flag(CpuFlag::Carry) {
            self.register_assign(RegisterId::Ip, address);
        }

        Ok(())
    }

    pub(super) fn JNC(&mut self) -> InstructionResult {
        let address = get_effective_address(self);

        if self.get_flag(CpuFlag::Carry) == false {
            self.register_assign(RegisterId::Ip, address);
        }

        Ok(())
    }

    pub(super) fn JBE(&mut self) -> InstructionResult {
        let address = get_effective_address(self);

        if self.get_flag(CpuFlag::Carry) || self.get_flag(CpuFlag::Zero) {
            self.register_assign(RegisterId::Ip, address);
        }

        Ok(())
    }

    pub(super) fn JA(&mut self) -> InstructionResult {
        let address = get_effective_address(self);

        if self.get_flag(CpuFlag::Carry) == false || self.get_flag(CpuFlag::Zero) == false {
            self.register_assign(RegisterId::Ip, address);
        }

        Ok(())
    }

    pub(super) fn JL(&mut self) -> InstructionResult {
        let address = get_effective_address(self);

        if self.get_flag(CpuFlag::Negative) != self.get_flag(CpuFlag::Overflow) {
            self.register_assign(RegisterId::Ip, address);
        }

        Ok(())
    }

    pub(super) fn JGE(&mut self) -> InstructionResult {
        let address = get_effective_address(self);

        if self.get_flag(CpuFlag::Negative) == self.get_flag(CpuFlag::Overflow) {
            self.register_assign(RegisterId::Ip, address);
        }

        Ok(())
    }

    pub(super) fn JLE(&mut self) -> InstructionResult {
        let address = get_effective_address(self);

        if self.get_flag(CpuFlag::Zero)
            || self.get_flag(CpuFlag::Negative) != self.get_flag(CpuFlag::Overflow)
        {
            self.register_assign(RegisterId::Ip, address);
        }

        Ok(())
    }

    pub(super) fn JG(&mut self) -> InstructionResult {
        let address = get_effective_address(self);

        if self.get_flag(CpuFlag::Zero) == false
            && self.get_flag(CpuFlag::Negative) == self.get_flag(CpuFlag::Overflow)
        {
            self.register_assign(RegisterId::Ip, address);
        }

        Ok(())
    }

    pub(super) fn CALL(&mut self) -> InstructionResult {
        let address = get_effective_address(self);

        self.push_qword(self.register(RegisterId::Ip));

        self.register_assign(RegisterId::Ip, address);

        Ok(())
    }

    pub(super) fn RET(&mut self) -> InstructionResult {
        let return_address = self.pop_qword();

        self.register_assign(RegisterId::Ip, return_address);

        Ok(())
    }

    pub(super) fn LIDT(&mut self) -> InstructionResult {
        let address = get_effective_address(self);

        self.idt = address;

        Ok(())
    }

    pub(super) fn RETI(&mut self) -> InstructionResult {
        let address = self.pop_qword();
        self.pop_flags();
        self.register_assign(RegisterId::Ip, address);

        Ok(())
    }

    pub(super) fn INT(&mut self) -> InstructionResult {
        let idt_entry = self.fetch_byte();
        self.interrupt_request(idt_entry);

        Ok(())
    }

    pub(super) fn CLI(&mut self) -> InstructionResult {
        self.set_flag(CpuFlag::InterruptEnable, false);

        Ok(())
    }

    pub(super) fn STI(&mut self) -> InstructionResult {
        self.set_flag(CpuFlag::InterruptEnable, true);

        Ok(())
    }

    pub(super) fn IN(&mut self) -> InstructionResult {
        let fetched_byte = self.fetch_byte();

        let dst_id: RegisterId = match RegisterId::from_u8(fetched_byte & 0b111) {
            Some(reg_id) => reg_id,
            None => return Err(INVALID_INSTRUCTION),
        };

        let port = self.fetch_word();

        let value = self.port_bus_read(port);

        self.register_assign(dst_id, value);

        Ok(())
    }

    pub(super) fn OUT(&mut self) -> InstructionResult {
        let fetched_byte = self.fetch_byte();

        let src_id: RegisterId = match RegisterId::from_u8(fetched_byte & 0b111) {
            Some(reg_id) => reg_id,
            None => return Err(INVALID_INSTRUCTION),
        };

        let port = self.fetch_word();

        self.port_bus_write(port, self.register(src_id));

        Ok(())
    }

    pub(super) fn NOP(&mut self) -> InstructionResult {
        Ok(())
    }
}
