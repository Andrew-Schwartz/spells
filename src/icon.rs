//! Selected boostrap icons. Machine generated code. Do not change!

/// Icons
#[derive(Copy, Clone, Debug, Hash)]
pub enum Icon {
	/// arrow-up
	ArrowUp,
	/// arrow-down
	ArrowDown,
	/// arrow-left
	ArrowLeft,
	/// arrow-right
	ArrowRight,
	/// arrow-clockwise
	ArrowClockwise,
	/// arrows-collapse
	ArrowsCollapse,
	/// arrows-expand
	ArrowsExpand,
	/// diamond-fill
	DiamondFill,
	/// diamond
	Diamond,
	/// trash
	Trash,
	/// check
	Check,
	/// check2
	Check2,
	/// x
	X,
	/// brightness-high
	BrightnessHigh,
	/// moon
	Moon,
	/// chevron-expand
	ChevronExpand,
	/// chevron-contract
	ChevronContract,
	/// info-circle
	InfoCircle,
	/// archive
	Archive,
}

/// Converts an icon into a char.
#[must_use]
#[allow(clippy::too_many_lines)]
pub const fn icon_to_char(icon: Icon) -> char {
	match icon {
		Icon::ArrowUp => '\u{61}',
		Icon::ArrowDown => '\u{62}',
		Icon::ArrowLeft => '\u{63}',
		Icon::ArrowRight => '\u{64}',
		Icon::ArrowClockwise => '\u{65}',
		Icon::ArrowsCollapse => '\u{66}',
		Icon::ArrowsExpand => '\u{67}',
		Icon::DiamondFill => '\u{68}',
		Icon::Diamond => '\u{69}',
		Icon::Trash => '\u{6a}',
		Icon::Check => '\u{6b}',
		Icon::Check2 => '\u{6c}',
		Icon::X => '\u{6d}',
		Icon::BrightnessHigh => '\u{6e}',
		Icon::Moon => '\u{6f}',
		Icon::ChevronExpand => '\u{70}',
		Icon::ChevronContract => '\u{71}',
		Icon::InfoCircle => '\u{72}',
		Icon::Archive => '\u{73}',
	}
}

impl std::fmt::Display for Icon {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use std::fmt::Write;
		f.write_char(icon_to_char(*self))
	}
}
