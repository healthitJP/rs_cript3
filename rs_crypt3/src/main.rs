use std::collections::HashMap;
#[macro_use]
extern crate lazy_static;
use std::sync::Mutex;

lazy_static! {
        /* The c and d arrays used to calculate the key schedule. */
        static ref C: Mutex<[u8; 28]> = Mutex::new([0u8; 28]);
        static ref D: Mutex<[u8; 28]> = Mutex::new([0u8; 28]);
        /* The key schedule.  Generated from the key. */
        static ref KS: Mutex<[[u8; 48]; 16]> = Mutex::new([[0u8; 48]; 16]);
        static ref E: Mutex<[u8; 48]> = Mutex::new([0u8; 48]); 
        /* The combination of the key and the input, before selection. */   
        static ref preS: Mutex<[u8; 48]> = Mutex::new([0u8; 48]);         
}
/* Initial permutation */
const IP: [i8; 64] = [
        58, 50, 42, 34, 26, 18, 10, 2,
        60, 52, 44, 36, 28, 20, 12, 4,
        62, 54, 46, 38, 30, 22, 14, 6,
        64, 56, 48, 40, 32, 24, 16, 8,
        57, 49, 41, 33, 25, 17, 9, 1,
        59, 51, 43, 35, 27, 19, 11, 3,
        61, 53, 45, 37, 29, 21, 13, 5,
        63, 55, 47, 39, 31, 23, 15, 7,
];


// fn ip_permutation(input: &[u8; 64]) -> [u8; 64] {
//     let ip = [
//         58, 50, 42, 34, 26, 18, 10, 2,
//         60, 52, 44, 36, 28, 20, 12, 4,
//         62, 54, 46, 38, 30, 22, 14, 6,
//         64, 56, 48, 40, 32, 24, 16, 8,
//         57, 49, 41, 33, 25, 17, 9, 1,
//         59, 51, 43, 35, 27, 19, 11, 3,
//         61, 53, 45, 37, 29, 21, 13, 5,
//         63, 55, 47, 39, 31, 23, 15, 7,
//     ];

//     let mut output = [0u8; 64];
//     for i in 0..64 {
//         output[i] = input[ip[i] as usize - 1];
//     }
//     output
// }

/* Final permutation, FP = IP^(-1) */
const FP: [i8;64] = [
        40, 8, 48, 16, 56, 24, 64, 32,
        39, 7, 47, 15, 55, 23, 63, 31,
        38, 6, 46, 14, 54, 22, 62, 30,
        37, 5, 45, 13, 53, 21, 61, 29,
        36, 4, 44, 12, 52, 20, 60, 28,
        35, 3, 43, 11, 51, 19, 59, 27,
        34, 2, 42, 10, 50, 18, 58, 26,
        33, 1, 41, 9, 49, 17, 57, 25,
];

// fn fp_permutation(input: &[u8; 64]) -> [u8; 64] {
//     let fp = [
//         40, 8, 48, 16, 56, 24, 64, 32,
//         39, 7, 47, 15, 55, 23, 63, 31,
//         38, 6, 46, 14, 54, 22, 62, 30,
//         37, 5, 45, 13, 53, 21, 61, 29,
//         36, 4, 44, 12, 52, 20, 60, 28,
//         35, 3, 43, 11, 51, 19, 59, 27,
//         34, 2, 42, 10, 50, 18, 58, 26,
//         33, 1, 41, 9, 49, 17, 57, 25,
//     ];

//     let mut output = [0u8; 64];
//     for i in 0..64 {
//         output[i] = input[fp[i] as usize - 1];
//     }
//     output
// }

/**************************************************************************
* Permuted-choice 1 from the key bits to yield C and D.
* Note that bits 8,16... are left out:
* They are intended for a parity check.
**************************************************************************/
const PC1_C:[u8;28] = [
        57, 49, 41, 33, 25, 17, 9,
        1, 58, 50, 42, 34, 26, 18,
        10, 2, 59, 51, 43, 35, 27,
        19, 11, 3, 60, 52, 44, 36,
];

const PC1_D:[u8;28] = [
        63, 55, 47, 39, 31, 23, 15,
        7, 62, 54, 46, 38, 30, 22,
        14, 6, 61, 53, 45, 37, 29,
        21, 13, 5, 28, 20, 12, 4,
];

/* Sequence of shifts used for the key schedule. */
const SHIFTS:[u8;16] = [1, 1, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 1];

/**************************************************************************
* Permuted-choice 2, to pick out the bits from the CD array that generate
* the key schedule.
**************************************************************************/
const PC2_C:[u8;24] = [
        14, 17, 11, 24, 1, 5,
        3, 28, 15, 6, 21, 10,
        23, 19, 12, 4, 26, 8,
        16, 7, 27, 20, 13, 2,
];

const PC2_D:[u8;24] = [
        41, 52, 31, 37, 47, 55,
        30, 40, 51, 45, 33, 48,
        44, 49, 39, 56, 34, 53,
        46, 42, 50, 36, 29, 32,
];


const E2:[u8; 48] = [
        32, 1, 2, 3, 4, 5,
        4, 5, 6, 7, 8, 9,
        8, 9, 10, 11, 12, 13,
        12, 13, 14, 15, 16, 17,
        16, 17, 18, 19, 20, 21,
        20, 21, 22, 23, 24, 25,
        24, 25, 26, 27, 28, 29,
        28, 29, 30, 31, 32, 1,
];

/**************************************************************************
* Function:    setkey
*
* Description: Set up the key schedule from the encryption key.
*
* Inputs:      char *key
*              pointer to 64 character array.  Each character represents a
*              bit in the key.
*
* Returns:     none
**************************************************************************/
fn setkey(key: [u8; 66]){
        let mut c=C.lock().unwrap();
        let mut d=D.lock().unwrap();
        let mut ks=KS.lock().unwrap();
        let mut e=E.lock().unwrap();
        /**********************************************************************
         * First, generate C and D by permuting the key.  The low order bit of
         * each 8-bit char is not used, so C and D are only 28 bits apiece.
         **********************************************************************/
        for i in 0..28 {
                c[i] = key[PC1_C[i] as usize - 1];
                d[i] = key[PC1_D[i] as usize - 1];
        }
        /**********************************************************************
        * To generate Ki, rotate C and D according to schedule and pick up a
        * permutation using PC2.
        **********************************************************************/
        for i in 0..16 {
                for _ in 0..SHIFTS[i] {
                c.rotate_left(1);
                d.rotate_left(1);
                }

                for j in 0..24 {
                ks[i][j] = c[PC2_C[j] as usize - 1];
                ks[i][j + 24] = d[PC2_D[j] as usize -28 - 1];
                }
        }
        for i in 0..48{
                e[i]=E2[i];
        }
}
/**************************************************************************
 * The 8 selection functions. For some reason, they give a 0-origin
 * index, unlike everything else.
 **************************************************************************/
const S:[[u8;64];8]=[
    [
      14, 4, 13, 1, 2, 15, 11, 8, 3, 10, 6, 12, 5, 9, 0, 7, 0, 15, 7, 4, 14, 2,
      13, 1, 10, 6, 12, 11, 9, 5, 3, 8, 4, 1, 14, 8, 13, 6, 2, 11, 15, 12, 9, 7,
      3, 10, 5, 0, 15, 12, 8, 2, 4, 9, 1, 7, 5, 11, 3, 14, 10, 0, 6, 13,
    ],

    [
      15, 1, 8, 14, 6, 11, 3, 4, 9, 7, 2, 13, 12, 0, 5, 10, 3, 13, 4, 7, 15, 2,
      8, 14, 12, 0, 1, 10, 6, 9, 11, 5, 0, 14, 7, 11, 10, 4, 13, 1, 5, 8, 12, 6,
      9, 3, 2, 15, 13, 8, 10, 1, 3, 15, 4, 2, 11, 6, 7, 12, 0, 5, 14, 9,
    ],

    [
      10, 0, 9, 14, 6, 3, 15, 5, 1, 13, 12, 7, 11, 4, 2, 8, 13, 7, 0, 9, 3, 4,
      6, 10, 2, 8, 5, 14, 12, 11, 15, 1, 13, 6, 4, 9, 8, 15, 3, 0, 11, 1, 2, 12,
      5, 10, 14, 7, 1, 10, 13, 0, 6, 9, 8, 7, 4, 15, 14, 3, 11, 5, 2, 12,
    ],

    [
      7, 13, 14, 3, 0, 6, 9, 10, 1, 2, 8, 5, 11, 12, 4, 15, 13, 8, 11, 5, 6, 15,
      0, 3, 4, 7, 2, 12, 1, 10, 14, 9, 10, 6, 9, 0, 12, 11, 7, 13, 15, 1, 3, 14,
      5, 2, 8, 4, 3, 15, 0, 6, 10, 1, 13, 8, 9, 4, 5, 11, 12, 7, 2, 14,
    ],

    [
      2, 12, 4, 1, 7, 10, 11, 6, 8, 5, 3, 15, 13, 0, 14, 9, 14, 11, 2, 12, 4, 7,
      13, 1, 5, 0, 15, 10, 3, 9, 8, 6, 4, 2, 1, 11, 10, 13, 7, 8, 15, 9, 12, 5,
      6, 3, 0, 14, 11, 8, 12, 7, 1, 14, 2, 13, 6, 15, 0, 9, 10, 4, 5, 3,
    ],

    [
      12, 1, 10, 15, 9, 2, 6, 8, 0, 13, 3, 4, 14, 7, 5, 11, 10, 15, 4, 2, 7, 12,
      9, 5, 6, 1, 13, 14, 0, 11, 3, 8, 9, 14, 15, 5, 2, 8, 12, 3, 7, 0, 4, 10,
      1, 13, 11, 6, 4, 3, 2, 12, 9, 5, 15, 10, 11, 14, 1, 7, 6, 0, 8, 13,
    ],

    [
      4, 11, 2, 14, 15, 0, 8, 13, 3, 12, 9, 7, 5, 10, 6, 1, 13, 0, 11, 7, 4, 9,
      1, 10, 14, 3, 5, 12, 2, 15, 8, 6, 1, 4, 11, 13, 12, 3, 7, 14, 10, 15, 6,
      8, 0, 5, 9, 2, 6, 11, 13, 8, 1, 4, 10, 7, 9, 5, 0, 15, 14, 2, 3, 12,
    ],

    [
      13, 2, 8, 4, 6, 15, 11, 1, 10, 9, 3, 14, 5, 0, 12, 7, 1, 15, 13, 8, 10, 3,
      7, 4, 12, 5, 6, 11, 0, 14, 9, 2, 7, 11, 4, 1, 9, 12, 14, 2, 0, 6, 10, 13,
      15, 3, 5, 8, 2, 1, 14, 7, 4, 10, 8, 13, 15, 12, 9, 0, 3, 5, 6, 11,
    ],
];
/**************************************************************************
 * P is a permutation on the selected combination of the current L and key.
 **************************************************************************/
const P:[u8;32] = [
16, 7, 20, 21, 29, 12, 28, 17, 1, 15, 23, 26, 5, 18, 31, 10, 2, 8, 24, 14,
32, 27, 3, 9, 19, 13, 30, 6, 22, 11, 4, 25,
];
/**************************************************************************
 * Function:    encrypt
 *
 * Description: Uses DES to encrypt a 64 bit block of data.  Requires
 *              setkey to be invoked with the encryption key before it may
 *              be used.  The results of the encryption are stored in block.
 *
 * Inputs:      char *block
 *              pointer to 64 character array.  Each character represents a
 *              bit in the data block.
 *
 * Returns:     none
 **************************************************************************/
fn encrypt(block: &mut [u8; 66]) {
        let mut left = [0u8; 32];
        let mut right = [0u8; 32];/* block in two halves */
        let mut old = [0u8; 32];
        let mut f = [0u8; 32];

        let mut pre_s=preS.lock().unwrap();
        let e=E.lock().unwrap();
        let ks=KS.lock().unwrap();


        /* First, permute the bits in the input */
        for j in 0..32 {
                left[j] = block[IP[j] as usize - 1];
        }

        for j in 32..64 {
                right[j - 32] = block[IP[j] as usize - 1];
        }

        /* Perform an encryption operation 16 times. */
        for i in 0..16 {
                /* Save the right array, which will be the new left. */
                old.copy_from_slice(&right);
                /******************************************************************
                 * Expand right to 48 bits using the E selector and
                 * exclusive-or with the current key bits.
                 ******************************************************************/
                for j in 0..48 {
                        pre_s[j] = right[e[j] as usize - 1] ^ ks[i][j];
                }

                for j in 0..8 {
                        let temp = 6 * j;
                        let k = S[j][
                                ((pre_s[temp] as usize) << 5)+
                                ((pre_s[temp + 1] as usize) << 3) +
                                ((pre_s[temp + 2] as usize) << 2) +
                                ((pre_s[temp + 3] as usize) << 1) +
                                ((pre_s[temp + 4] as usize) << 0) +
                                ((pre_s[temp + 5] as usize) << 4)
                        ];
                        let temp = 4 * j;
                        f[temp] = (k >> 3) & 1;
                        f[temp + 1] = (k >> 2) & 1;
                        f[temp + 2] = (k >> 1) & 1;
                        f[temp + 3] = k & 1;
                }
                /******************************************************************
                 * The new right is left ^ f(R, K).
                 * The f here has to be permuted first, though.
                 ******************************************************************/
                for j in 0..32 {
                        right[j] = left[j] ^ f[P[j] as usize - 1];
                }

                left.copy_from_slice(&old);
        }

        for j in 0..32 {
                let temp = left[j];
                left[j] = right[j];
                right[j] = temp;
        }

        for j in 0..64 {
                let i = FP[j];
                block[j] = if i < 33 { left[FP[j] as usize - 1] } else { right[FP[j] as usize - 33] };
        }
        }
        /**************************************************************************
         * Function:    crypt
         *
         * Description: Clone of Unix crypt(3) function.
         *
         * Inputs:      char *pw
         *              pointer to 8 character encryption key (user password)
         *              char *salt
         *              pointer to 2 character salt used to modify the DES results.
         *
         * Returns:     Pointer to static array containing the salt concatenated
         *              on to the encrypted results.  Same as stored in passwd file.
         **************************************************************************/
fn crypt(pw: &str, salt: &str) -> String {
        let mut e =E.lock().unwrap();
        if salt.len() != 2 {
                panic!("Length of salt must be 2 (!= {})", salt.len());
        }

        let mut block = [0u8; 66]; /* 1st store key, then results */
        let mut iobuf = [0u8; 16]; /* encrypted results */

        /* break pw into 64 bits */
        let pw_bytes = pw.as_bytes();
        let mut _c = 0;
        let mut _i = 0;

        while _c < pw_bytes.len() && _i < 64 {
                for _j in 0..7 {
                        if _i >= 64 {
                                break;
                        }
                        block[_i] = (pw_bytes[_c] >> (6 - _j)) & 1;
                        _i += 1;
                }
                _c += 1;
                _i += 1;
        }

        /* set key based on pw */       
        setkey(block);

        for byte in &mut block {
                *byte = 0;
        }

        let mut _c = 0;
        for i in 0..2 {
                iobuf[i] = salt.as_bytes()[_c];
                let mut k = salt.as_bytes()[_c];
                _c += 1;

                if k > b'Z' {
                k -= 6;
                }

                if k > b'9' {
                k -= 7;
                }

                k -= b'.';

                for j in 0..6 {
                        if (k >> j) & 1 != 0 {
                                let temp = e[6 * i + j];
                                e[6 * i + j] = e[6 * i + j + 24];
                                e[6 * i + j + 24] = temp;
                        }
                }
        }

        for _ in 0..25 {
                encrypt(&mut block);
        }

        for i in 0..11 {
                let mut c = 0;
                for j in 0..6 {
                        c <<= 1;
                        c |= block[6 * i + j];
                }

                c += b'.';
                if c > b'9' {
                        c += 7;
                }

                if c > b'Z' {
                        c += 6;
                }

                iobuf[i + 2] = c;
        }
        
        iobuf[11+2]=0;

        /* prevent premature NULL terminator */
        if iobuf[1] == 0 {
                iobuf[1] = iobuf[0];
        }

        iobuf.iter().take_while(|&&byte| byte != 0).map(|&byte| byte as char).collect()
        // std::str::from_utf8(&iobuf[..i + 2]).expect("UTF-8エラー");
}

fn main() {
    println!("Hello, world!");
}
