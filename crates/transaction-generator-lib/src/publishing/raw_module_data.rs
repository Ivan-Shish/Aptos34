// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

// This file was generated. Do not modify!
//
// To update this code, run `cargo run -p module-publish` in aptos core.
// That test compiles the set of modules defined in
// `testsuite/simple/src/simple/sources/`
// and it writes the binaries here.
// The module name (prefixed with `MODULE_`) is a `Lazy` instance that returns the
// byte array of the module binary.
// This crate should also provide a Rust file that allows proper manipulation of each
// module defined below.

use std::collections::HashMap;

use once_cell::sync::Lazy;

#[rustfmt::skip]
pub static PACKAGE_SIMPLE_METADATA: Lazy<Vec<u8>> = Lazy::new(|| {
	vec![
		13, 71, 101, 110, 101, 114, 105, 99, 77, 111, 100, 117, 108, 101, 1, 0, 0, 0,
		0, 0, 0, 0, 0, 64, 48, 57, 69, 67, 52, 51, 56, 49, 48, 57, 48, 70,
		52, 57, 49, 51, 54, 57, 67, 65, 52, 56, 51, 57, 48, 55, 54, 68, 66, 70,
		67, 65, 53, 55, 51, 56, 57, 53, 52, 56, 53, 65, 57, 52, 52, 51, 55, 52,
		66, 65, 55, 55, 50, 55, 49, 56, 70, 54, 70, 66, 49, 50, 49, 48, 132, 1,
		31, 139, 8, 0, 0, 0, 0, 0, 2, 255, 77, 139, 59, 14, 194, 48, 16, 68,
		251, 61, 133, 229, 30, 135, 11, 80, 208, 64, 197, 9, 162, 20, 43, 123, 64, 86,
		156, 93, 203, 134, 80, 32, 238, 142, 45, 1, 138, 102, 154, 249, 188, 49, 179, 159,
		249, 134, 137, 132, 23, 152, 131, 177, 103, 8, 74, 244, 23, 13, 143, 4, 75, 43,
		74, 141, 42, 125, 217, 187, 38, 75, 52, 6, 100, 72, 128, 248, 136, 58, 209, 49,
		223, 181, 158, 74, 195, 159, 90, 230, 118, 124, 153, 164, 158, 83, 71, 156, 27, 182,
		230, 126, 221, 45, 186, 98, 184, 254, 128, 111, 249, 207, 214, 188, 233, 3, 132, 221,
		66, 189, 150, 0, 0, 0, 1, 6, 115, 105, 109, 112, 108, 101, 0, 0, 0, 3,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 14, 65, 112, 116,
		111, 115, 70, 114, 97, 109, 101, 119, 111, 114, 107, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 1, 11, 65, 112, 116, 111, 115, 83, 116, 100, 108, 105,
		98, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 77, 111,
		118, 101, 83, 116, 100, 108, 105, 98, 0,
	]
});

#[rustfmt::skip]
pub static MODULE_SIMPLE_SIMPLE: Lazy<Vec<u8>> = Lazy::new(|| {
	vec![
		161, 28, 235, 11, 6, 0, 0, 0, 12, 1, 0, 12, 2, 12, 34, 3, 46, 132,
		1, 4, 178, 1, 4, 5, 182, 1, 197, 1, 7, 251, 2, 210, 3, 8, 205, 6,
		64, 6, 141, 7, 115, 16, 128, 8, 62, 10, 190, 8, 42, 12, 232, 8, 235, 11,
		13, 211, 20, 14, 0, 0, 1, 1, 1, 2, 1, 3, 1, 4, 1, 5, 0, 6,
		8, 0, 0, 7, 8, 0, 0, 8, 7, 0, 0, 9, 8, 0, 0, 10, 8, 0,
		0, 11, 6, 0, 5, 22, 7, 0, 3, 37, 4, 1, 6, 1, 0, 12, 0, 1,
		0, 0, 13, 2, 1, 0, 0, 14, 3, 4, 0, 0, 15, 5, 1, 0, 0, 16,
		6, 1, 0, 0, 17, 5, 1, 0, 0, 18, 6, 1, 0, 0, 19, 5, 1, 0,
		0, 20, 5, 1, 0, 0, 21, 6, 1, 0, 0, 23, 7, 1, 0, 0, 24, 8,
		1, 0, 0, 25, 8, 1, 0, 0, 26, 5, 1, 0, 0, 27, 9, 1, 0, 0,
		28, 10, 1, 0, 0, 29, 5, 1, 0, 0, 30, 6, 1, 0, 0, 31, 11, 1,
		0, 0, 32, 8, 1, 0, 0, 33, 5, 1, 0, 4, 41, 5, 15, 0, 5, 42,
		18, 19, 0, 1, 43, 5, 22, 1, 6, 3, 44, 23, 1, 1, 6, 2, 45, 12,
		12, 0, 23, 21, 24, 21, 2, 7, 10, 2, 6, 10, 2, 0, 2, 6, 12, 10,
		2, 4, 6, 8, 4, 6, 8, 4, 6, 8, 1, 6, 8, 1, 1, 6, 3, 1,
		6, 12, 2, 6, 12, 3, 4, 6, 12, 3, 8, 6, 10, 2, 2, 6, 12, 5,
		2, 6, 12, 6, 12, 5, 6, 12, 6, 12, 6, 12, 6, 12, 6, 12, 2, 6,
		12, 8, 6, 1, 3, 1, 2, 2, 7, 8, 0, 8, 0, 1, 5, 3, 6, 3,
		6, 3, 6, 3, 3, 3, 8, 4, 7, 8, 4, 1, 10, 2, 1, 8, 6, 2,
		7, 8, 3, 5, 1, 8, 5, 1, 11, 7, 1, 9, 0, 2, 7, 11, 7, 1,
		9, 0, 9, 0, 3, 10, 3, 10, 3, 3, 3, 8, 2, 7, 8, 4, 8, 4,
		8, 1, 10, 2, 7, 8, 4, 10, 2, 3, 3, 8, 4, 7, 8, 4, 9, 3,
		7, 8, 4, 3, 3, 3, 8, 4, 7, 8, 4, 6, 8, 4, 6, 8, 4, 2,
		7, 8, 4, 8, 4, 2, 8, 4, 7, 8, 4, 3, 7, 8, 1, 7, 8, 1,
		3, 1, 7, 8, 1, 6, 115, 105, 109, 112, 108, 101, 7, 97, 99, 99, 111, 117,
		110, 116, 5, 101, 114, 114, 111, 114, 5, 101, 118, 101, 110, 116, 6, 115, 105, 103,
		110, 101, 114, 6, 115, 116, 114, 105, 110, 103, 12, 66, 121, 116, 101, 82, 101, 115,
		111, 117, 114, 99, 101, 7, 67, 111, 117, 110, 116, 101, 114, 4, 68, 97, 116, 97,
		10, 69, 118, 101, 110, 116, 83, 116, 111, 114, 101, 8, 82, 101, 115, 111, 117, 114,
		99, 101, 11, 83, 105, 109, 112, 108, 101, 69, 118, 101, 110, 116, 11, 97, 112, 112,
		101, 110, 100, 95, 100, 97, 116, 97, 20, 98, 121, 116, 101, 115, 95, 109, 97, 107,
		101, 95, 111, 114, 95, 99, 104, 97, 110, 103, 101, 14, 99, 111, 112, 121, 95, 112,
		97, 115, 116, 97, 95, 114, 101, 102, 6, 100, 111, 117, 98, 108, 101, 11, 101, 109,
		105, 116, 95, 101, 118, 101, 110, 116, 115, 11, 103, 101, 116, 95, 99, 111, 117, 110,
		116, 101, 114, 21, 103, 101, 116, 95, 102, 114, 111, 109, 95, 114, 97, 110, 100, 111,
		109, 95, 99, 111, 110, 115, 116, 4, 104, 97, 108, 102, 11, 105, 110, 105, 116, 95,
		109, 111, 100, 117, 108, 101, 5, 108, 111, 111, 112, 121, 6, 83, 116, 114, 105, 110,
		103, 14, 109, 97, 107, 101, 95, 111, 114, 95, 99, 104, 97, 110, 103, 101, 8, 109,
		97, 120, 105, 109, 105, 122, 101, 8, 109, 105, 110, 105, 109, 105, 122, 101, 3, 110,
		111, 112, 13, 110, 111, 112, 95, 50, 95, 115, 105, 103, 110, 101, 114, 115, 13, 110,
		111, 112, 95, 53, 95, 115, 105, 103, 110, 101, 114, 115, 10, 114, 101, 115, 101, 116,
		95, 100, 97, 116, 97, 6, 115, 101, 116, 95, 105, 100, 8, 115, 101, 116, 95, 110,
		97, 109, 101, 16, 115, 116, 101, 112, 95, 100, 101, 115, 116, 105, 110, 97, 116, 105,
		111, 110, 11, 115, 116, 101, 112, 95, 115, 105, 103, 110, 101, 114, 4, 100, 97, 116,
		97, 5, 99, 111, 117, 110, 116, 13, 115, 105, 109, 112, 108, 101, 95, 101, 118, 101,
		110, 116, 115, 11, 69, 118, 101, 110, 116, 72, 97, 110, 100, 108, 101, 2, 105, 100,
		4, 110, 97, 109, 101, 8, 101, 118, 101, 110, 116, 95, 105, 100, 10, 97, 100, 100,
		114, 101, 115, 115, 95, 111, 102, 4, 117, 116, 102, 56, 16, 110, 101, 119, 95, 101,
		118, 101, 110, 116, 95, 104, 97, 110, 100, 108, 101, 10, 101, 109, 105, 116, 95, 101,
		118, 101, 110, 116, 16, 105, 110, 118, 97, 108, 105, 100, 95, 97, 114, 103, 117, 109,
		101, 110, 116, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 171, 205, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 8, 1, 0, 0,
		0, 0, 0, 0, 0, 10, 2, 9, 8, 1, 35, 69, 103, 137, 171, 205, 239, 10,
		2, 6, 5, 104, 101, 108, 108, 111, 10, 3, 81, 10, 0, 0, 0, 0, 0, 0,
		0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0,
		3, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 5, 0,
		0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0,
		0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0, 0, 0,
		0, 0, 18, 97, 112, 116, 111, 115, 58, 58, 109, 101, 116, 97, 100, 97, 116, 97,
		95, 118, 49, 42, 1, 1, 0, 0, 0, 0, 0, 0, 0, 29, 69, 67, 79, 85,
		78, 84, 69, 82, 95, 82, 69, 83, 79, 85, 82, 67, 69, 95, 78, 79, 84, 95,
		80, 82, 69, 83, 69, 78, 84, 0, 0, 0, 0, 2, 1, 34, 10, 2, 1, 2,
		1, 35, 3, 2, 2, 1, 34, 10, 2, 3, 2, 1, 36, 11, 7, 1, 8, 5,
		4, 2, 3, 38, 3, 39, 8, 6, 34, 8, 2, 5, 2, 1, 40, 3, 0, 0,
		0, 0, 12, 26, 10, 1, 65, 13, 12, 2, 10, 2, 6, 0, 0, 0, 0, 0,
		0, 0, 0, 36, 4, 21, 5, 8, 10, 0, 10, 1, 10, 2, 6, 1, 0, 0,
		0, 0, 0, 0, 0, 23, 66, 13, 20, 68, 13, 11, 2, 6, 1, 0, 0, 0,
		0, 0, 0, 0, 23, 12, 2, 5, 3, 11, 1, 1, 11, 0, 1, 2, 1, 1,
		4, 1, 0, 14, 20, 10, 0, 17, 21, 41, 0, 4, 13, 11, 0, 17, 21, 42,
		0, 12, 2, 11, 1, 11, 2, 15, 0, 21, 5, 19, 11, 1, 18, 0, 12, 3,
		11, 0, 11, 3, 45, 0, 2, 2, 0, 0, 0, 16, 103, 10, 0, 16, 1, 12,
		5, 10, 1, 16, 1, 12, 6, 11, 5, 20, 10, 6, 20, 35, 4, 18, 11, 6,
		12, 5, 10, 2, 16, 2, 12, 6, 5, 26, 11, 6, 1, 10, 1, 16, 1, 12,
		5, 10, 3, 16, 2, 12, 6, 10, 6, 20, 10, 1, 16, 1, 20, 35, 4, 47,
		11, 5, 1, 11, 1, 1, 11, 0, 1, 11, 2, 1, 11, 6, 12, 5, 11, 3,
		16, 2, 12, 6, 5, 69, 11, 3, 1, 10, 5, 11, 0, 16, 1, 34, 4, 65,
		11, 6, 1, 11, 5, 1, 11, 2, 16, 2, 12, 5, 11, 1, 16, 1, 12, 6,
		5, 69, 11, 1, 1, 11, 2, 1, 10, 5, 20, 10, 6, 20, 35, 4, 82, 11,
		6, 1, 10, 5, 12, 6, 10, 5, 1, 5, 88, 11, 5, 1, 10, 6, 12, 5,
		10, 6, 1, 10, 5, 10, 6, 33, 4, 97, 11, 6, 1, 11, 5, 12, 4, 5,
		101, 11, 5, 1, 11, 6, 12, 4, 11, 4, 2, 3, 1, 4, 1, 4, 17, 44,
		10, 0, 17, 21, 41, 4, 32, 4, 16, 6, 0, 0, 0, 0, 0, 0, 0, 0,
		7, 2, 17, 22, 7, 1, 18, 2, 18, 4, 12, 2, 11, 0, 11, 2, 45, 4,
		5, 43, 11, 0, 17, 21, 42, 4, 12, 3, 10, 3, 16, 3, 16, 4, 65, 13,
		6, 2, 0, 0, 0, 0, 0, 0, 0, 24, 12, 1, 10, 3, 16, 3, 16, 4,
		65, 13, 10, 1, 35, 4, 41, 5, 35, 10, 3, 15, 3, 15, 4, 49, 255, 68,
		13, 5, 27, 11, 3, 1, 2, 4, 0, 0, 1, 3, 20, 36, 10, 0, 17, 21,
		12, 3, 10, 3, 41, 3, 32, 4, 13, 10, 0, 11, 0, 56, 0, 18, 3, 45,
		3, 5, 15, 11, 0, 1, 11, 3, 42, 3, 12, 2, 10, 1, 6, 0, 0, 0,
		0, 0, 0, 0, 0, 36, 4, 33, 5, 23, 11, 1, 6, 1, 0, 0, 0, 0,
		0, 0, 0, 23, 12, 1, 10, 2, 15, 5, 10, 1, 18, 5, 56, 1, 5, 18,
		11, 2, 1, 2, 5, 1, 4, 1, 1, 1, 7, 11, 0, 17, 21, 43, 1, 16,
		2, 20, 1, 2, 6, 1, 4, 0, 24, 25, 7, 3, 12, 2, 14, 2, 65, 12,
		12, 4, 10, 4, 6, 0, 0, 0, 0, 0, 0, 0, 0, 34, 4, 24, 10, 1,
		10, 4, 38, 4, 17, 11, 4, 6, 1, 0, 0, 0, 0, 0, 0, 0, 23, 12,
		1, 7, 3, 12, 3, 14, 3, 11, 1, 66, 12, 20, 1, 2, 7, 1, 4, 1,
		4, 17, 44, 10, 0, 17, 21, 41, 4, 32, 4, 16, 6, 0, 0, 0, 0, 0,
		0, 0, 0, 7, 2, 17, 22, 7, 1, 18, 2, 18, 4, 12, 2, 11, 0, 11,
		2, 45, 4, 5, 43, 11, 0, 17, 21, 42, 4, 12, 3, 10, 3, 16, 3, 16,
		4, 65, 13, 6, 2, 0, 0, 0, 0, 0, 0, 0, 26, 12, 1, 10, 3, 16,
		3, 16, 4, 65, 13, 10, 1, 36, 4, 41, 5, 35, 10, 3, 15, 3, 15, 4,
		69, 13, 1, 5, 27, 11, 3, 1, 2, 8, 0, 0, 0, 1, 5, 11, 0, 6,
		0, 0, 0, 0, 0, 0, 0, 0, 18, 1, 45, 1, 2, 9, 1, 4, 0, 1,
		11, 10, 1, 6, 0, 0, 0, 0, 0, 0, 0, 0, 36, 4, 10, 5, 5, 11,
		1, 6, 1, 0, 0, 0, 0, 0, 0, 0, 23, 12, 1, 5, 0, 2, 10, 1,
		4, 1, 4, 25, 34, 10, 0, 17, 21, 41, 4, 4, 22, 11, 0, 17, 21, 42,
		4, 12, 5, 11, 1, 10, 5, 15, 1, 21, 11, 2, 10, 5, 15, 6, 21, 11,
		3, 11, 5, 15, 3, 15, 4, 21, 5, 33, 11, 3, 18, 2, 12, 4, 11, 1,
		11, 2, 11, 4, 18, 4, 12, 6, 11, 0, 11, 6, 45, 4, 2, 11, 1, 4,
		1, 4, 26, 93, 10, 1, 41, 4, 4, 6, 11, 0, 1, 2, 10, 0, 17, 21,
		41, 4, 32, 4, 21, 6, 0, 0, 0, 0, 0, 0, 0, 0, 7, 2, 17, 22,
		7, 1, 18, 2, 18, 4, 12, 8, 10, 0, 11, 8, 45, 4, 10, 0, 17, 21,
		43, 4, 16, 3, 16, 4, 65, 13, 12, 6, 10, 1, 43, 4, 16, 3, 16, 4,
		65, 13, 12, 7, 11, 6, 11, 7, 36, 4, 49, 11, 0, 17, 21, 43, 4, 16,
		3, 16, 4, 20, 11, 1, 42, 4, 12, 4, 12, 3, 5, 59, 11, 1, 43, 4,
		16, 3, 16, 4, 20, 11, 0, 17, 21, 42, 4, 12, 4, 12, 3, 11, 3, 11,
		4, 12, 9, 12, 5, 14, 5, 65, 13, 10, 9, 16, 3, 16, 4, 65, 13, 36,
		4, 75, 5, 72, 8, 12, 2, 5, 82, 10, 9, 16, 3, 16, 4, 65, 13, 6,
		16, 39, 0, 0, 0, 0, 0, 0, 35, 12, 2, 11, 2, 4, 90, 10, 9, 15,
		3, 15, 4, 14, 5, 17, 0, 5, 63, 11, 9, 1, 2, 12, 1, 4, 1, 4,
		27, 81, 10, 1, 41, 4, 4, 6, 11, 0, 1, 2, 10, 0, 17, 21, 41, 4,
		32, 4, 21, 6, 0, 0, 0, 0, 0, 0, 0, 0, 7, 2, 17, 22, 7, 1,
		18, 2, 18, 4, 12, 7, 10, 0, 11, 7, 45, 4, 10, 0, 17, 21, 43, 4,
		12, 9, 10, 1, 43, 4, 12, 10, 11, 9, 16, 3, 16, 4, 65, 13, 11, 10,
		16, 3, 16, 4, 65, 13, 12, 5, 12, 4, 10, 4, 10, 5, 36, 4, 51, 11,
		5, 6, 2, 0, 0, 0, 0, 0, 0, 0, 26, 11, 0, 17, 21, 42, 4, 12,
		3, 12, 2, 5, 60, 11, 0, 1, 11, 4, 6, 2, 0, 0, 0, 0, 0, 0,
		0, 26, 11, 1, 42, 4, 12, 3, 12, 2, 11, 2, 11, 3, 12, 8, 12, 6,
		10, 8, 16, 3, 16, 4, 65, 13, 10, 6, 36, 4, 78, 5, 72, 10, 8, 15,
		3, 15, 4, 69, 13, 1, 5, 64, 11, 8, 1, 2, 13, 1, 4, 0, 1, 1,
		2, 14, 1, 4, 0, 1, 1, 2, 15, 1, 4, 0, 1, 1, 2, 16, 1, 4,
		1, 4, 28, 34, 10, 0, 17, 21, 41, 4, 4, 23, 11, 0, 17, 21, 42, 4,
		12, 1, 6, 0, 0, 0, 0, 0, 0, 0, 0, 10, 1, 15, 1, 21, 7, 2,
		17, 22, 10, 1, 15, 6, 21, 7, 1, 11, 1, 15, 3, 15, 4, 21, 5, 33,
		6, 0, 0, 0, 0, 0, 0, 0, 0, 7, 2, 17, 22, 7, 1, 18, 2, 18,
		4, 12, 2, 11, 0, 11, 2, 45, 4, 2, 17, 1, 4, 1, 4, 29, 25, 10,
		0, 17, 21, 41, 4, 32, 4, 16, 11, 1, 7, 2, 17, 22, 7, 1, 18, 2,
		18, 4, 12, 2, 11, 0, 11, 2, 45, 4, 5, 24, 11, 0, 17, 21, 42, 4,
		12, 3, 11, 1, 11, 3, 15, 1, 21, 2, 18, 1, 4, 1, 4, 29, 24, 10,
		0, 17, 21, 41, 4, 32, 4, 15, 6, 0, 0, 0, 0, 0, 0, 0, 0, 11,
		1, 7, 1, 18, 2, 18, 4, 12, 2, 11, 0, 11, 2, 45, 4, 5, 23, 11,
		0, 17, 21, 42, 4, 12, 3, 11, 1, 11, 3, 15, 6, 21, 2, 19, 1, 4,
		1, 1, 30, 42, 10, 1, 41, 1, 4, 4, 5, 9, 11, 0, 1, 7, 0, 17,
		25, 39, 11, 1, 42, 1, 12, 2, 10, 2, 16, 2, 20, 7, 0, 22, 10, 2,
		15, 2, 21, 11, 2, 16, 2, 20, 12, 4, 10, 0, 17, 21, 41, 1, 4, 37,
		11, 0, 17, 21, 42, 1, 12, 3, 11, 4, 11, 3, 15, 2, 21, 5, 41, 11,
		0, 11, 4, 18, 1, 45, 1, 2, 20, 1, 4, 1, 1, 31, 13, 11, 0, 17,
		21, 42, 1, 12, 1, 10, 1, 16, 2, 20, 7, 0, 22, 11, 1, 15, 2, 21,
		2, 0, 0, 4, 0, 1, 0, 4, 2, 2, 0, 3, 0, 4, 1, 0,
	]
});

#[rustfmt::skip]
pub static MODULES_SIMPLE: Lazy<Vec<Vec<u8>>> = Lazy::new(|| { vec![
	MODULE_SIMPLE_SIMPLE.to_vec(),
]});

#[rustfmt::skip]
pub static PACKAGE_FRAMEWORK_USECASES_METADATA: Lazy<Vec<u8>> = Lazy::new(|| {
	vec![
		17, 70, 114, 97, 109, 101, 119, 111, 114, 107, 85, 115, 101, 99, 97, 115, 101, 115,
		1, 0, 0, 0, 0, 0, 0, 0, 0, 64, 65, 65, 66, 53, 65, 68, 54, 50,
		69, 57, 48, 57, 67, 54, 66, 48, 68, 56, 66, 51, 68, 55, 55, 67, 56, 48,
		54, 65, 51, 48, 70, 48, 66, 57, 67, 56, 48, 68, 48, 56, 68, 66, 70, 57,
		49, 56, 48, 67, 70, 54, 68, 51, 69, 65, 48, 54, 52, 69, 53, 69, 57, 68,
		54, 49, 156, 1, 31, 139, 8, 0, 0, 0, 0, 0, 2, 255, 165, 140, 59, 14,
		194, 48, 16, 68, 123, 159, 194, 114, 31, 135, 11, 80, 208, 208, 210, 64, 21, 165,
		48, 206, 128, 130, 19, 175, 229, 181, 66, 129, 184, 59, 182, 2, 17, 180, 68, 187,
		197, 126, 230, 189, 38, 24, 235, 204, 21, 173, 240, 102, 132, 220, 74, 181, 143, 121,
		184, 83, 116, 39, 134, 53, 12, 86, 98, 66, 228, 158, 124, 249, 110, 116, 46, 37,
		68, 211, 33, 192, 119, 240, 182, 7, 183, 98, 23, 18, 241, 66, 230, 224, 67, 14,
		100, 205, 80, 16, 173, 235, 239, 54, 37, 90, 141, 52, 161, 190, 124, 128, 247, 113,
		217, 149, 124, 206, 202, 35, 57, 248, 127, 117, 169, 192, 191, 170, 195, 249, 6, 155,
		120, 149, 177, 162, 89, 82, 204, 47, 183, 88, 117, 215, 63, 1, 0, 0, 1, 8,
		116, 111, 107, 101, 110, 95, 118, 49, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 1, 14, 65, 112, 116, 111, 115, 70, 114, 97, 109,
		101, 119, 111, 114, 107, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		1, 11, 65, 112, 116, 111, 115, 83, 116, 100, 108, 105, 98, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 77, 111, 118, 101, 83, 116, 100, 108,
		105, 98, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 10, 65,
		112, 116, 111, 115, 84, 111, 107, 101, 110, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 4, 17, 65, 112, 116, 111, 115, 84, 111, 107, 101, 110, 79, 98,
		106, 101, 99, 116, 115, 0,
	]
});

#[rustfmt::skip]
pub static MODULE_FRAMEWORK_USECASES_TOKEN_V1: Lazy<Vec<u8>> = Lazy::new(|| {
	vec![
		161, 28, 235, 11, 6, 0, 0, 0, 11, 1, 0, 16, 2, 16, 42, 3, 58, 167,
		1, 4, 225, 1, 12, 5, 237, 1, 139, 2, 7, 248, 3, 185, 6, 8, 177, 10,
		96, 6, 145, 11, 150, 1, 10, 167, 12, 19, 12, 186, 12, 185, 5, 13, 243, 17,
		6, 0, 0, 1, 1, 1, 2, 1, 3, 1, 4, 1, 5, 1, 6, 2, 7, 0,
		8, 8, 0, 4, 9, 7, 0, 7, 12, 7, 0, 7, 15, 4, 0, 7, 17, 7,
		0, 1, 27, 6, 0, 6, 29, 4, 2, 3, 1, 0, 1, 2, 30, 7, 1, 0,
		0, 7, 44, 7, 0, 0, 10, 0, 1, 0, 0, 11, 2, 3, 0, 0, 13, 4,
		5, 0, 0, 14, 2, 5, 0, 0, 16, 6, 7, 0, 0, 18, 8, 9, 0, 0,
		19, 10, 7, 0, 0, 20, 4, 7, 0, 0, 21, 4, 7, 0, 0, 22, 4, 7,
		0, 0, 23, 4, 7, 0, 0, 24, 4, 7, 0, 0, 25, 4, 7, 0, 4, 32,
		11, 7, 0, 5, 33, 13, 1, 1, 0, 4, 34, 14, 7, 0, 1, 35, 15, 3,
		0, 3, 36, 10, 2, 0, 1, 37, 2, 12, 0, 4, 38, 17, 1, 0, 7, 39,
		18, 19, 0, 7, 40, 21, 22, 0, 2, 41, 23, 24, 1, 0, 2, 42, 24, 26,
		1, 0, 6, 43, 28, 7, 2, 3, 0, 7, 45, 30, 31, 0, 7, 46, 32, 9,
		0, 1, 47, 35, 36, 0, 7, 48, 37, 7, 0, 6, 49, 7, 38, 2, 3, 4,
		7, 50, 40, 25, 0, 7, 51, 43, 7, 0, 14, 12, 14, 2, 22, 12, 23, 25,
		24, 27, 29, 27, 2, 8, 1, 3, 1, 8, 1, 1, 5, 1, 12, 2, 6, 12,
		5, 2, 12, 8, 2, 3, 5, 5, 8, 3, 0, 4, 6, 12, 8, 1, 8, 1,
		3, 1, 8, 4, 1, 6, 12, 2, 7, 8, 1, 10, 2, 1, 3, 1, 6, 9,
		0, 2, 7, 8, 1, 8, 1, 1, 6, 8, 5, 5, 5, 12, 8, 2, 8, 1,
		8, 4, 1, 10, 2, 3, 6, 12, 8, 4, 3, 1, 8, 2, 6, 11, 7, 1,
		3, 3, 12, 8, 2, 8, 4, 8, 1, 2, 5, 8, 1, 1, 11, 7, 1, 3,
		1, 7, 11, 7, 1, 9, 0, 1, 9, 0, 1, 8, 3, 1, 11, 7, 1, 9,
		0, 2, 5, 11, 7, 1, 8, 3, 3, 7, 11, 6, 2, 9, 0, 9, 1, 9,
		0, 9, 1, 5, 10, 1, 8, 1, 5, 8, 8, 8, 1, 1, 6, 10, 1, 1,
		8, 8, 13, 6, 12, 8, 1, 8, 1, 8, 1, 3, 8, 1, 5, 3, 3, 8,
		8, 10, 8, 1, 10, 10, 2, 10, 8, 1, 6, 8, 1, 8, 1, 8, 1, 12,
		8, 5, 8, 4, 1, 2, 2, 6, 12, 10, 2, 2, 12, 8, 5, 6, 6, 12,
		8, 1, 8, 1, 8, 1, 3, 10, 1, 1, 11, 6, 2, 9, 0, 9, 1, 5,
		12, 6, 12, 8, 3, 8, 2, 8, 4, 3, 6, 12, 8, 2, 3, 3, 12, 8,
		3, 8, 2, 4, 12, 6, 12, 8, 2, 8, 4, 4, 6, 12, 6, 12, 8, 2,
		3, 8, 116, 111, 107, 101, 110, 95, 118, 49, 7, 97, 99, 99, 111, 117, 110, 116,
		6, 111, 112, 116, 105, 111, 110, 6, 115, 105, 103, 110, 101, 114, 6, 115, 116, 114,
		105, 110, 103, 12, 115, 116, 114, 105, 110, 103, 95, 117, 116, 105, 108, 115, 5, 116,
		97, 98, 108, 101, 5, 116, 111, 107, 101, 110, 12, 77, 105, 110, 116, 101, 114, 67,
		111, 110, 102, 105, 103, 6, 83, 116, 114, 105, 110, 103, 16, 98, 117, 105, 108, 100,
		95, 116, 111, 107, 101, 110, 95, 110, 97, 109, 101, 10, 103, 101, 116, 95, 115, 105,
		103, 110, 101, 114, 7, 84, 111, 107, 101, 110, 73, 100, 17, 109, 105, 110, 116, 95,
		110, 102, 116, 95, 112, 97, 114, 97, 108, 108, 101, 108, 19, 109, 105, 110, 116, 95,
		110, 102, 116, 95, 115, 101, 113, 117, 101, 110, 116, 105, 97, 108, 5, 84, 111, 107,
		101, 110, 16, 115, 101, 116, 95, 116, 111, 107, 101, 110, 95, 109, 105, 110, 116, 101,
		100, 11, 84, 111, 107, 101, 110, 68, 97, 116, 97, 73, 100, 26, 116, 111, 107, 101,
		110, 95, 118, 49, 95, 99, 114, 101, 97, 116, 101, 95, 116, 111, 107, 101, 110, 95,
		100, 97, 116, 97, 30, 116, 111, 107, 101, 110, 95, 118, 49, 95, 105, 110, 105, 116,
		105, 97, 108, 105, 122, 101, 95, 99, 111, 108, 108, 101, 99, 116, 105, 111, 110, 26,
		116, 111, 107, 101, 110, 95, 118, 49, 95, 109, 105, 110, 116, 95, 97, 110, 100, 95,
		115, 116, 111, 114, 101, 95, 102, 116, 36, 116, 111, 107, 101, 110, 95, 118, 49, 95,
		109, 105, 110, 116, 95, 97, 110, 100, 95, 115, 116, 111, 114, 101, 95, 110, 102, 116,
		95, 112, 97, 114, 97, 108, 108, 101, 108, 38, 116, 111, 107, 101, 110, 95, 118, 49,
		95, 109, 105, 110, 116, 95, 97, 110, 100, 95, 115, 116, 111, 114, 101, 95, 110, 102,
		116, 95, 115, 101, 113, 117, 101, 110, 116, 105, 97, 108, 29, 116, 111, 107, 101, 110,
		95, 118, 49, 95, 109, 105, 110, 116, 95, 97, 110, 100, 95, 116, 114, 97, 110, 115,
		102, 101, 114, 95, 102, 116, 39, 116, 111, 107, 101, 110, 95, 118, 49, 95, 109, 105,
		110, 116, 95, 97, 110, 100, 95, 116, 114, 97, 110, 115, 102, 101, 114, 95, 110, 102,
		116, 95, 112, 97, 114, 97, 108, 108, 101, 108, 41, 116, 111, 107, 101, 110, 95, 118,
		49, 95, 109, 105, 110, 116, 95, 97, 110, 100, 95, 116, 114, 97, 110, 115, 102, 101,
		114, 95, 110, 102, 116, 95, 115, 101, 113, 117, 101, 110, 116, 105, 97, 108, 10, 115,
		105, 103, 110, 101, 114, 95, 99, 97, 112, 16, 83, 105, 103, 110, 101, 114, 67, 97,
		112, 97, 98, 105, 108, 105, 116, 121, 13, 109, 105, 110, 116, 101, 100, 95, 116, 111,
		107, 101, 110, 115, 5, 84, 97, 98, 108, 101, 6, 79, 112, 116, 105, 111, 110, 12,
		116, 111, 107, 101, 110, 100, 97, 116, 97, 95, 105, 100, 11, 97, 112, 112, 101, 110,
		100, 95, 117, 116, 102, 56, 9, 116, 111, 95, 115, 116, 114, 105, 110, 103, 6, 97,
		112, 112, 101, 110, 100, 29, 99, 114, 101, 97, 116, 101, 95, 115, 105, 103, 110, 101,
		114, 95, 119, 105, 116, 104, 95, 99, 97, 112, 97, 98, 105, 108, 105, 116, 121, 10,
		97, 100, 100, 114, 101, 115, 115, 95, 111, 102, 19, 103, 101, 116, 95, 115, 101, 113,
		117, 101, 110, 99, 101, 95, 110, 117, 109, 98, 101, 114, 4, 117, 116, 102, 56, 10,
		109, 105, 110, 116, 95, 116, 111, 107, 101, 110, 21, 103, 101, 116, 95, 99, 111, 108,
		108, 101, 99, 116, 105, 111, 110, 95, 115, 117, 112, 112, 108, 121, 7, 101, 120, 116,
		114, 97, 99, 116, 4, 115, 111, 109, 101, 3, 97, 100, 100, 21, 84, 111, 107, 101,
		110, 77, 117, 116, 97, 98, 105, 108, 105, 116, 121, 67, 111, 110, 102, 105, 103, 30,
		99, 114, 101, 97, 116, 101, 95, 116, 111, 107, 101, 110, 95, 109, 117, 116, 97, 98,
		105, 108, 105, 116, 121, 95, 99, 111, 110, 102, 105, 103, 16, 99, 114, 101, 97, 116,
		101, 95, 116, 111, 107, 101, 110, 100, 97, 116, 97, 23, 99, 114, 101, 97, 116, 101,
		95, 114, 101, 115, 111, 117, 114, 99, 101, 95, 97, 99, 99, 111, 117, 110, 116, 17,
		99, 114, 101, 97, 116, 101, 95, 99, 111, 108, 108, 101, 99, 116, 105, 111, 110, 3,
		110, 101, 119, 14, 119, 105, 116, 104, 100, 114, 97, 119, 95, 116, 111, 107, 101, 110,
		15, 100, 105, 114, 101, 99, 116, 95, 116, 114, 97, 110, 115, 102, 101, 114, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 171, 205, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 3, 10, 2, 30, 29, 65, 110, 32, 78, 70, 84, 32, 67, 111, 108,
		108, 101, 99, 116, 105, 111, 110, 32, 68, 101, 115, 99, 114, 105, 112, 116, 105, 111,
		110, 10, 2, 23, 22, 65, 110, 32, 78, 70, 84, 32, 67, 111, 108, 108, 101, 99,
		116, 105, 111, 110, 32, 78, 97, 109, 101, 10, 2, 1, 0, 3, 8, 100, 0, 0,
		0, 0, 0, 0, 0, 3, 8, 0, 0, 0, 0, 0, 0, 0, 0, 10, 2, 16,
		15, 78, 70, 84, 32, 67, 111, 108, 108, 101, 99, 116, 105, 98, 108, 101, 10, 2,
		18, 17, 104, 116, 116, 112, 115, 58, 47, 47, 97, 112, 116, 111, 115, 46, 100, 101,
		118, 10, 2, 3, 2, 32, 35, 10, 1, 6, 5, 1, 1, 1, 1, 1, 10, 10,
		2, 1, 0, 10, 1, 4, 3, 0, 0, 0, 0, 2, 3, 26, 8, 5, 28, 11,
		6, 2, 5, 11, 7, 1, 8, 3, 31, 8, 4, 0, 0, 0, 0, 7, 9, 13,
		0, 7, 7, 17, 13, 13, 0, 14, 1, 56, 0, 17, 15, 11, 0, 2, 1, 0,
		0, 1, 0, 7, 5, 11, 0, 43, 0, 16, 0, 17, 16, 2, 2, 0, 0, 1,
		0, 16, 28, 11, 1, 17, 1, 12, 3, 10, 0, 17, 17, 12, 2, 14, 2, 56,
		1, 11, 0, 17, 17, 17, 18, 17, 0, 12, 5, 14, 3, 7, 6, 17, 19, 11,
		5, 6, 1, 0, 0, 0, 0, 0, 0, 0, 17, 5, 12, 6, 14, 3, 11, 6,
		6, 1, 0, 0, 0, 0, 0, 0, 0, 17, 20, 12, 4, 11, 3, 11, 4, 2,
		3, 0, 0, 1, 0, 20, 34, 11, 0, 17, 1, 12, 3, 14, 3, 17, 17, 7,
		1, 17, 19, 17, 21, 12, 1, 13, 1, 56, 2, 6, 1, 0, 0, 0, 0, 0,
		0, 0, 22, 12, 2, 7, 5, 17, 19, 11, 2, 17, 0, 12, 6, 14, 3, 7,
		6, 17, 19, 11, 6, 6, 1, 0, 0, 0, 0, 0, 0, 0, 17, 5, 12, 5,
		14, 3, 11, 5, 6, 1, 0, 0, 0, 0, 0, 0, 0, 17, 20, 12, 4, 11,
		3, 11, 4, 2, 4, 0, 0, 1, 0, 7, 8, 11, 1, 42, 0, 15, 1, 11,
		0, 11, 2, 56, 3, 56, 4, 2, 5, 0, 0, 0, 29, 29, 7, 1, 17, 19,
		12, 5, 7, 2, 17, 19, 12, 8, 10, 0, 17, 17, 12, 6, 7, 8, 12, 4,
		14, 4, 17, 25, 12, 7, 11, 0, 11, 5, 11, 2, 11, 8, 11, 3, 11, 1,
		11, 6, 7, 3, 7, 4, 11, 7, 64, 1, 0, 0, 0, 0, 0, 0, 0, 0,
		7, 9, 64, 1, 0, 0, 0, 0, 0, 0, 0, 0, 17, 26, 2, 6, 1, 4,
		0, 33, 36, 10, 0, 64, 34, 0, 0, 0, 0, 0, 0, 0, 0, 17, 27, 12,
		5, 12, 4, 7, 1, 17, 19, 12, 1, 7, 0, 17, 19, 12, 3, 7, 2, 17,
		19, 12, 2, 14, 4, 11, 1, 11, 3, 11, 2, 6, 64, 66, 15, 0, 0, 0,
		0, 0, 7, 10, 17, 28, 14, 4, 7, 6, 17, 19, 7, 5, 17, 19, 6, 64,
		75, 76, 0, 0, 0, 0, 0, 17, 5, 12, 6, 11, 0, 11, 5, 56, 5, 11,
		6, 18, 0, 45, 0, 2, 7, 1, 4, 1, 0, 39, 26, 10, 1, 17, 1, 12,
		2, 14, 2, 12, 3, 10, 1, 43, 0, 16, 2, 20, 12, 6, 10, 3, 11, 6,
		6, 1, 0, 0, 0, 0, 0, 0, 0, 17, 20, 12, 5, 11, 3, 11, 5, 6,
		1, 0, 0, 0, 0, 0, 0, 0, 17, 30, 12, 4, 11, 0, 17, 17, 11, 1,
		11, 4, 17, 4, 2, 8, 1, 4, 1, 0, 41, 16, 10, 0, 10, 1, 17, 2,
		12, 4, 12, 2, 14, 2, 11, 4, 6, 1, 0, 0, 0, 0, 0, 0, 0, 17,
		30, 12, 3, 11, 0, 17, 17, 11, 1, 11, 3, 17, 4, 2, 9, 1, 4, 1,
		0, 41, 15, 10, 1, 17, 3, 12, 4, 12, 2, 14, 2, 11, 4, 6, 1, 0,
		0, 0, 0, 0, 0, 0, 17, 30, 12, 3, 11, 0, 17, 17, 11, 1, 11, 3,
		17, 4, 2, 10, 1, 4, 1, 0, 42, 21, 10, 1, 17, 1, 12, 2, 14, 2,
		12, 3, 11, 1, 43, 0, 16, 2, 20, 12, 5, 10, 3, 11, 5, 6, 1, 0,
		0, 0, 0, 0, 0, 0, 17, 20, 12, 4, 11, 3, 11, 0, 11, 4, 6, 1,
		0, 0, 0, 0, 0, 0, 0, 17, 31, 2, 11, 1, 4, 1, 0, 5, 11, 10,
		0, 11, 1, 17, 2, 12, 3, 12, 2, 14, 2, 11, 0, 11, 3, 6, 1, 0,
		0, 0, 0, 0, 0, 0, 17, 31, 2, 12, 1, 4, 1, 0, 5, 10, 11, 1,
		17, 3, 12, 3, 12, 2, 14, 2, 11, 0, 11, 3, 6, 1, 0, 0, 0, 0,
		0, 0, 0, 17, 31, 2, 0, 0, 0, 1, 0, 2, 0,
	]
});

#[rustfmt::skip]
pub static MODULES_FRAMEWORK_USECASES: Lazy<Vec<Vec<u8>>> = Lazy::new(|| { vec![
	MODULE_FRAMEWORK_USECASES_TOKEN_V1.to_vec(),
]});
#[rustfmt::skip]
pub static PACKAGE_AMBASSADOR_TOKEN_METADATA: Lazy<Vec<u8>> = Lazy::new(|| {
	vec![
		16, 97, 109, 98, 97, 115, 115, 97, 100, 111, 114, 95, 116, 111, 107, 101, 110, 1,
		0, 0, 0, 0, 0, 0, 0, 0, 64, 57, 52, 54, 48, 56, 66, 55, 69, 66,
		56, 50, 69, 55, 57, 66, 48, 70, 65, 57, 67, 57, 49, 48, 48, 51, 55, 66,
		49, 70, 65, 56, 52, 54, 66, 70, 50, 57, 52, 68, 55, 48, 51, 50, 57, 48,
		51, 65, 50, 69, 48, 50, 49, 51, 68, 48, 54, 56, 67, 50, 52, 55, 49, 49,
		51, 173, 1, 31, 139, 8, 0, 0, 0, 0, 0, 2, 255, 141, 142, 191, 14, 130,
		48, 16, 198, 247, 62, 69, 195, 194, 68, 193, 7, 112, 32, 70, 86, 23, 55, 66,
		200, 209, 158, 6, 129, 30, 233, 17, 53, 49, 190, 187, 45, 106, 103, 146, 91, 46,
		223, 239, 251, 83, 207, 160, 7, 184, 98, 35, 44, 76, 40, 247, 50, 133, 169, 3,
		102, 48, 228, 218, 133, 6, 180, 169, 184, 163, 227, 158, 108, 16, 119, 170, 80, 69,
		42, 68, 13, 198, 56, 100, 70, 110, 196, 74, 181, 212, 221, 80, 47, 236, 161, 164,
		120, 30, 202, 234, 152, 120, 202, 224, 140, 214, 160, 213, 125, 0, 203, 121, 33, 174,
		156, 239, 121, 144, 27, 60, 249, 146, 35, 105, 24, 131, 71, 169, 60, 222, 229, 143,
		228, 16, 28, 89, 252, 19, 249, 254, 134, 156, 67, 229, 41, 54, 110, 204, 89, 135,
		102, 191, 161, 33, 235, 3, 233, 216, 40, 40, 253, 0, 0, 0, 1, 10, 97, 109,
		98, 97, 115, 115, 97, 100, 111, 114, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 1, 14, 65, 112, 116, 111, 115, 70, 114, 97, 109,
		101, 119, 111, 114, 107, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		1, 11, 65, 112, 116, 111, 115, 83, 116, 100, 108, 105, 98, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 77, 111, 118, 101, 83, 116, 100, 108,
		105, 98, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 17, 65,
		112, 116, 111, 115, 84, 111, 107, 101, 110, 79, 98, 106, 101, 99, 116, 115, 0,
	]
});

#[rustfmt::skip]
pub static MODULE_AMBASSADOR_TOKEN_AMBASSADOR: Lazy<Vec<u8>> = Lazy::new(|| {
	vec![
		161, 28, 235, 11, 6, 0, 0, 0, 12, 1, 0, 22, 2, 22, 62, 3, 84, 184,
		1, 4, 140, 2, 18, 5, 158, 2, 160, 2, 7, 190, 4, 225, 6, 8, 159, 11,
		96, 6, 255, 11, 197, 1, 16, 196, 13, 178, 3, 10, 246, 16, 27, 12, 145, 17,
		135, 4, 13, 152, 21, 6, 0, 0, 1, 1, 1, 2, 1, 3, 1, 4, 1, 5,
		1, 6, 2, 7, 2, 8, 2, 9, 2, 10, 0, 11, 8, 0, 0, 12, 8, 0,
		0, 13, 6, 0, 3, 14, 7, 1, 0, 1, 6, 16, 7, 0, 10, 26, 6, 0,
		8, 28, 6, 0, 2, 30, 4, 1, 6, 1, 9, 41, 11, 0, 4, 42, 7, 1,
		0, 0, 3, 44, 2, 0, 8, 46, 10, 0, 3, 47, 6, 0, 3, 53, 2, 0,
		0, 15, 0, 1, 0, 0, 17, 0, 2, 0, 0, 18, 3, 4, 0, 0, 19, 5,
		4, 0, 0, 20, 5, 4, 0, 0, 21, 6, 4, 0, 0, 22, 7, 4, 0, 0,
		23, 8, 4, 0, 0, 24, 9, 4, 0, 3, 33, 11, 12, 1, 8, 6, 34, 14,
		2, 0, 8, 35, 15, 2, 1, 8, 1, 36, 1, 1, 0, 10, 37, 17, 12, 1,
		8, 5, 38, 5, 12, 0, 1, 39, 1, 1, 0, 2, 40, 19, 4, 1, 6, 8,
		18, 20, 4, 0, 10, 18, 21, 4, 0, 4, 43, 4, 24, 1, 0, 7, 45, 25,
		26, 0, 10, 48, 28, 26, 0, 3, 49, 29, 30, 0, 3, 50, 29, 31, 0, 10,
		51, 29, 21, 0, 8, 52, 29, 20, 0, 3, 54, 32, 33, 0, 3, 55, 34, 4,
		0, 3, 56, 32, 4, 0, 8, 57, 35, 36, 0, 8, 58, 37, 4, 0, 8, 59,
		38, 4, 1, 2, 3, 60, 5, 19, 1, 6, 2, 61, 40, 4, 1, 6, 8, 62,
		42, 4, 1, 2, 9, 10, 11, 10, 13, 10, 16, 18, 19, 23, 31, 2, 32, 18,
		33, 18, 34, 2, 1, 11, 3, 1, 8, 1, 1, 3, 1, 8, 4, 2, 6, 12,
		11, 3, 1, 8, 1, 0, 1, 6, 12, 5, 6, 12, 8, 4, 8, 4, 8, 4,
		5, 5, 6, 12, 6, 12, 8, 4, 8, 4, 8, 4, 3, 6, 12, 11, 3, 1,
		8, 1, 3, 2, 11, 3, 1, 8, 1, 3, 1, 8, 1, 1, 6, 11, 3, 1,
		9, 0, 1, 5, 2, 8, 4, 6, 11, 3, 1, 8, 1, 1, 10, 2, 2, 6,
		11, 3, 1, 9, 0, 6, 8, 4, 5, 8, 5, 6, 12, 11, 7, 1, 8, 2,
		8, 6, 6, 11, 3, 1, 8, 1, 1, 11, 3, 1, 9, 0, 1, 8, 2, 1,
		11, 7, 1, 9, 0, 1, 8, 6, 1, 8, 5, 3, 8, 4, 8, 4, 8, 4,
		1, 8, 8, 1, 11, 9, 1, 9, 0, 5, 6, 12, 8, 4, 8, 4, 11, 9,
		1, 8, 8, 8, 4, 1, 8, 10, 8, 8, 1, 8, 5, 8, 4, 8, 10, 12,
		8, 11, 8, 6, 8, 12, 6, 6, 12, 8, 4, 8, 4, 8, 4, 11, 9, 1,
		8, 8, 8, 4, 1, 6, 8, 10, 1, 12, 1, 8, 12, 1, 6, 8, 12, 1,
		8, 13, 2, 8, 13, 5, 3, 10, 8, 4, 10, 8, 4, 10, 10, 2, 1, 8,
		11, 2, 6, 8, 10, 8, 11, 3, 6, 8, 6, 8, 4, 9, 0, 4, 7, 8,
		0, 6, 12, 6, 11, 3, 1, 8, 1, 5, 2, 7, 11, 7, 1, 9, 0, 9,
		0, 5, 10, 2, 10, 2, 8, 4, 6, 8, 6, 10, 2, 3, 6, 8, 6, 6,
		8, 4, 9, 0, 10, 97, 109, 98, 97, 115, 115, 97, 100, 111, 114, 5, 101, 114,
		114, 111, 114, 5, 101, 118, 101, 110, 116, 6, 111, 98, 106, 101, 99, 116, 6, 111,
		112, 116, 105, 111, 110, 6, 115, 105, 103, 110, 101, 114, 6, 115, 116, 114, 105, 110,
		103, 10, 99, 111, 108, 108, 101, 99, 116, 105, 111, 110, 12, 112, 114, 111, 112, 101,
		114, 116, 121, 95, 109, 97, 112, 7, 114, 111, 121, 97, 108, 116, 121, 5, 116, 111,
		107, 101, 110, 15, 65, 109, 98, 97, 115, 115, 97, 100, 111, 114, 76, 101, 118, 101,
		108, 15, 65, 109, 98, 97, 115, 115, 97, 100, 111, 114, 84, 111, 107, 101, 110, 16,
		76, 101, 118, 101, 108, 85, 112, 100, 97, 116, 101, 69, 118, 101, 110, 116, 6, 79,
		98, 106, 101, 99, 116, 16, 97, 109, 98, 97, 115, 115, 97, 100, 111, 114, 95, 108,
		101, 118, 101, 108, 6, 83, 116, 114, 105, 110, 103, 15, 97, 109, 98, 97, 115, 115,
		97, 100, 111, 114, 95, 114, 97, 110, 107, 4, 98, 117, 114, 110, 28, 99, 114, 101,
		97, 116, 101, 95, 97, 109, 98, 97, 115, 115, 97, 100, 111, 114, 95, 99, 111, 108,
		108, 101, 99, 116, 105, 111, 110, 11, 105, 110, 105, 116, 95, 109, 111, 100, 117, 108,
		101, 21, 109, 105, 110, 116, 95, 97, 109, 98, 97, 115, 115, 97, 100, 111, 114, 95,
		116, 111, 107, 101, 110, 29, 109, 105, 110, 116, 95, 97, 109, 98, 97, 115, 115, 97,
		100, 111, 114, 95, 116, 111, 107, 101, 110, 95, 98, 121, 95, 117, 115, 101, 114, 20,
		115, 101, 116, 95, 97, 109, 98, 97, 115, 115, 97, 100, 111, 114, 95, 108, 101, 118,
		101, 108, 22, 117, 112, 100, 97, 116, 101, 95, 97, 109, 98, 97, 115, 115, 97, 100,
		111, 114, 95, 114, 97, 110, 107, 8, 98, 117, 114, 110, 95, 114, 101, 102, 7, 66,
		117, 114, 110, 82, 101, 102, 20, 112, 114, 111, 112, 101, 114, 116, 121, 95, 109, 117,
		116, 97, 116, 111, 114, 95, 114, 101, 102, 10, 77, 117, 116, 97, 116, 111, 114, 82,
		101, 102, 19, 108, 101, 118, 101, 108, 95, 117, 112, 100, 97, 116, 101, 95, 101, 118,
		101, 110, 116, 115, 11, 69, 118, 101, 110, 116, 72, 97, 110, 100, 108, 101, 9, 111,
		108, 100, 95, 108, 101, 118, 101, 108, 9, 110, 101, 119, 95, 108, 101, 118, 101, 108,
		14, 111, 98, 106, 101, 99, 116, 95, 97, 100, 100, 114, 101, 115, 115, 4, 117, 116,
		102, 56, 11, 114, 101, 97, 100, 95, 115, 116, 114, 105, 110, 103, 9, 110, 111, 116,
		95, 102, 111, 117, 110, 100, 7, 99, 114, 101, 97, 116, 111, 114, 10, 97, 100, 100,
		114, 101, 115, 115, 95, 111, 102, 17, 112, 101, 114, 109, 105, 115, 115, 105, 111, 110,
		95, 100, 101, 110, 105, 101, 100, 14, 100, 101, 115, 116, 114, 111, 121, 95, 104, 97,
		110, 100, 108, 101, 7, 82, 111, 121, 97, 108, 116, 121, 6, 79, 112, 116, 105, 111,
		110, 4, 110, 111, 110, 101, 14, 67, 111, 110, 115, 116, 114, 117, 99, 116, 111, 114,
		82, 101, 102, 27, 99, 114, 101, 97, 116, 101, 95, 117, 110, 108, 105, 109, 105, 116,
		101, 100, 95, 99, 111, 108, 108, 101, 99, 116, 105, 111, 110, 11, 80, 114, 111, 112,
		101, 114, 116, 121, 77, 97, 112, 11, 84, 114, 97, 110, 115, 102, 101, 114, 82, 101,
		102, 18, 99, 114, 101, 97, 116, 101, 95, 110, 97, 109, 101, 100, 95, 116, 111, 107,
		101, 110, 15, 103, 101, 110, 101, 114, 97, 116, 101, 95, 115, 105, 103, 110, 101, 114,
		21, 103, 101, 110, 101, 114, 97, 116, 101, 95, 116, 114, 97, 110, 115, 102, 101, 114,
		95, 114, 101, 102, 17, 103, 101, 110, 101, 114, 97, 116, 101, 95, 98, 117, 114, 110,
		95, 114, 101, 102, 20, 103, 101, 110, 101, 114, 97, 116, 101, 95, 109, 117, 116, 97,
		116, 111, 114, 95, 114, 101, 102, 17, 76, 105, 110, 101, 97, 114, 84, 114, 97, 110,
		115, 102, 101, 114, 82, 101, 102, 28, 103, 101, 110, 101, 114, 97, 116, 101, 95, 108,
		105, 110, 101, 97, 114, 95, 116, 114, 97, 110, 115, 102, 101, 114, 95, 114, 101, 102,
		17, 116, 114, 97, 110, 115, 102, 101, 114, 95, 119, 105, 116, 104, 95, 114, 101, 102,
		24, 100, 105, 115, 97, 98, 108, 101, 95, 117, 110, 103, 97, 116, 101, 100, 95, 116,
		114, 97, 110, 115, 102, 101, 114, 13, 112, 114, 101, 112, 97, 114, 101, 95, 105, 110,
		112, 117, 116, 4, 105, 110, 105, 116, 9, 97, 100, 100, 95, 116, 121, 112, 101, 100,
		16, 110, 101, 119, 95, 101, 118, 101, 110, 116, 95, 104, 97, 110, 100, 108, 101, 10,
		101, 109, 105, 116, 95, 101, 118, 101, 110, 116, 12, 117, 112, 100, 97, 116, 101, 95,
		116, 121, 112, 101, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 202,
		254, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 10, 2, 34, 33, 65, 109, 98,
		97, 115, 115, 97, 100, 111, 114, 32, 67, 111, 108, 108, 101, 99, 116, 105, 111, 110,
		32, 68, 101, 115, 99, 114, 105, 112, 116, 105, 111, 110, 10, 2, 27, 26, 65, 109,
		98, 97, 115, 115, 97, 100, 111, 114, 32, 67, 111, 108, 108, 101, 99, 116, 105, 111,
		110, 32, 78, 97, 109, 101, 10, 2, 26, 25, 65, 109, 98, 97, 115, 115, 97, 100,
		111, 114, 32, 67, 111, 108, 108, 101, 99, 116, 105, 111, 110, 32, 85, 82, 73, 3,
		8, 6, 0, 0, 0, 0, 0, 0, 0, 3, 8, 3, 0, 0, 0, 0, 0, 0,
		0, 3, 8, 2, 0, 0, 0, 0, 0, 0, 0, 3, 8, 5, 0, 0, 0, 0,
		0, 0, 0, 3, 8, 1, 0, 0, 0, 0, 0, 0, 0, 3, 8, 4, 0, 0,
		0, 0, 0, 0, 0, 10, 2, 7, 6, 66, 114, 111, 110, 122, 101, 10, 2, 5,
		4, 71, 111, 108, 100, 10, 2, 7, 6, 83, 105, 108, 118, 101, 114, 10, 2, 5,
		4, 82, 97, 110, 107, 10, 10, 2, 1, 0, 18, 97, 112, 116, 111, 115, 58, 58,
		109, 101, 116, 97, 100, 97, 116, 97, 95, 118, 49, 157, 3, 6, 1, 0, 0, 0,
		0, 0, 0, 0, 21, 69, 84, 79, 75, 69, 78, 95, 68, 79, 69, 83, 95, 78,
		79, 84, 95, 69, 88, 73, 83, 84, 24, 84, 104, 101, 32, 116, 111, 107, 101, 110,
		32, 100, 111, 101, 115, 32, 110, 111, 116, 32, 101, 120, 105, 115, 116, 2, 0, 0,
		0, 0, 0, 0, 0, 12, 69, 78, 79, 84, 95, 67, 82, 69, 65, 84, 79, 82,
		38, 84, 104, 101, 32, 112, 114, 111, 118, 105, 100, 101, 100, 32, 115, 105, 103, 110,
		101, 114, 32, 105, 115, 32, 110, 111, 116, 32, 116, 104, 101, 32, 99, 114, 101, 97,
		116, 111, 114, 3, 0, 0, 0, 0, 0, 0, 0, 18, 69, 70, 73, 69, 76, 68,
		95, 78, 79, 84, 95, 77, 85, 84, 65, 66, 76, 69, 38, 65, 116, 116, 101, 109,
		112, 116, 101, 100, 32, 116, 111, 32, 109, 117, 116, 97, 116, 101, 32, 97, 110, 32,
		105, 109, 109, 117, 116, 97, 98, 108, 101, 32, 102, 105, 101, 108, 100, 4, 0, 0,
		0, 0, 0, 0, 0, 19, 69, 84, 79, 75, 69, 78, 95, 78, 79, 84, 95, 66,
		85, 82, 78, 65, 66, 76, 69, 38, 65, 116, 116, 101, 109, 112, 116, 101, 100, 32,
		116, 111, 32, 98, 117, 114, 110, 32, 97, 32, 110, 111, 110, 45, 98, 117, 114, 110,
		97, 98, 108, 101, 32, 116, 111, 107, 101, 110, 5, 0, 0, 0, 0, 0, 0, 0,
		23, 69, 80, 82, 79, 80, 69, 82, 84, 73, 69, 83, 95, 78, 79, 84, 95, 77,
		85, 84, 65, 66, 76, 69, 54, 65, 116, 116, 101, 109, 112, 116, 101, 100, 32, 116,
		111, 32, 109, 117, 116, 97, 116, 101, 32, 97, 32, 112, 114, 111, 112, 101, 114, 116,
		121, 32, 109, 97, 112, 32, 116, 104, 97, 116, 32, 105, 115, 32, 110, 111, 116, 32,
		109, 117, 116, 97, 98, 108, 101, 6, 0, 0, 0, 0, 0, 0, 0, 26, 69, 67,
		79, 76, 76, 69, 67, 84, 73, 79, 78, 95, 68, 79, 69, 83, 95, 78, 79, 84,
		95, 69, 88, 73, 83, 84, 0, 0, 2, 15, 97, 109, 98, 97, 115, 115, 97, 100,
		111, 114, 95, 114, 97, 110, 107, 1, 1, 0, 16, 97, 109, 98, 97, 115, 115, 97,
		100, 111, 114, 95, 108, 101, 118, 101, 108, 1, 1, 0, 0, 2, 1, 15, 3, 1,
		2, 3, 25, 8, 5, 27, 8, 6, 29, 11, 7, 1, 8, 2, 2, 2, 2, 31,
		3, 32, 3, 0, 1, 0, 1, 0, 4, 6, 14, 0, 56, 0, 43, 0, 16, 0,
		20, 2, 1, 1, 0, 0, 13, 9, 14, 0, 12, 2, 7, 12, 17, 10, 12, 1,
		11, 2, 14, 1, 56, 1, 2, 2, 1, 4, 1, 1, 16, 41, 11, 0, 14, 1,
		12, 6, 12, 3, 10, 6, 56, 0, 41, 1, 4, 9, 5, 16, 11, 6, 1, 11,
		3, 1, 7, 7, 17, 12, 39, 11, 6, 20, 56, 2, 11, 3, 17, 14, 33, 4,
		24, 5, 27, 7, 5, 17, 15, 39, 14, 1, 56, 0, 44, 1, 19, 1, 12, 4,
		12, 5, 12, 2, 11, 4, 56, 3, 11, 5, 17, 17, 11, 2, 17, 18, 2, 3,
		0, 0, 0, 22, 17, 7, 0, 17, 10, 12, 1, 7, 1, 17, 10, 12, 2, 7,
		2, 17, 10, 12, 3, 11, 0, 11, 1, 11, 2, 56, 4, 11, 3, 17, 20, 1,
		2, 4, 0, 0, 0, 4, 3, 11, 0, 17, 3, 2, 5, 1, 4, 0, 27, 57,
		7, 1, 17, 10, 12, 7, 11, 0, 11, 7, 11, 1, 11, 2, 56, 4, 11, 3,
		17, 21, 12, 8, 14, 8, 17, 22, 12, 9, 14, 8, 17, 23, 12, 12, 14, 8,
		17, 24, 12, 6, 14, 8, 17, 25, 12, 11, 14, 12, 17, 26, 11, 4, 17, 27,
		14, 12, 17, 28, 14, 9, 6, 0, 0, 0, 0, 0, 0, 0, 0, 18, 0, 45,
		0, 64, 2, 0, 0, 0, 0, 0, 0, 0, 0, 64, 2, 0, 0, 0, 0, 0,
		0, 0, 0, 7, 13, 17, 29, 12, 10, 14, 8, 11, 10, 17, 30, 14, 11, 7,
		12, 17, 10, 7, 9, 17, 10, 56, 5, 11, 6, 11, 11, 14, 9, 56, 6, 18,
		1, 12, 5, 14, 9, 11, 5, 45, 1, 2, 6, 1, 4, 0, 4, 8, 11, 1,
		11, 2, 11, 3, 11, 4, 11, 0, 17, 14, 17, 5, 2, 7, 1, 4, 2, 0,
		1, 39, 50, 11, 0, 14, 1, 12, 5, 12, 4, 10, 5, 56, 0, 41, 1, 4,
		9, 5, 16, 11, 5, 1, 11, 4, 1, 7, 7, 17, 12, 39, 11, 5, 20, 56,
		2, 11, 4, 17, 14, 33, 4, 24, 5, 27, 7, 5, 17, 15, 39, 14, 1, 56,
		0, 12, 6, 10, 6, 42, 0, 12, 3, 11, 6, 42, 1, 15, 1, 10, 3, 16,
		0, 20, 10, 2, 18, 2, 56, 7, 10, 2, 11, 3, 15, 0, 21, 11, 1, 11,
		2, 17, 8, 2, 8, 0, 0, 1, 1, 41, 34, 10, 1, 6, 10, 0, 0, 0,
		0, 0, 0, 0, 35, 4, 7, 7, 9, 12, 3, 5, 18, 11, 1, 6, 20, 0,
		0, 0, 0, 0, 0, 0, 35, 4, 14, 7, 11, 12, 2, 5, 16, 7, 10, 12,
		2, 11, 2, 12, 3, 11, 3, 12, 6, 14, 0, 56, 0, 43, 1, 16, 2, 12,
		5, 7, 12, 17, 10, 12, 4, 11, 5, 14, 4, 11, 6, 17, 10, 56, 8, 2,
		0, 0, 1, 2, 1, 1, 0,
	]
});

#[rustfmt::skip]
pub static MODULES_AMBASSADOR_TOKEN: Lazy<Vec<Vec<u8>>> = Lazy::new(|| { vec![
	MODULE_AMBASSADOR_TOKEN_AMBASSADOR.to_vec(),
]});

#[rustfmt::skip]
pub static PACKAGE_TO_METADATA: Lazy<HashMap<String, Vec<u8>>> = Lazy::new(|| { HashMap::from([
	("simple".to_string(), PACKAGE_SIMPLE_METADATA.to_vec()),
	("framework_usecases".to_string(), PACKAGE_FRAMEWORK_USECASES_METADATA.to_vec()),
	("ambassador_token".to_string(), PACKAGE_AMBASSADOR_TOKEN_METADATA.to_vec()),
])});

#[rustfmt::skip]
pub static PACKAGE_TO_MODULES: Lazy<HashMap<String, Vec<Vec<u8>>>> = Lazy::new(|| { HashMap::from([
	("simple".to_string(), MODULES_SIMPLE.to_vec()),
	("framework_usecases".to_string(), MODULES_FRAMEWORK_USECASES.to_vec()),
	("ambassador_token".to_string(), MODULES_AMBASSADOR_TOKEN.to_vec()),
])});
