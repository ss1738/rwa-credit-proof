// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

// Groth16 / BN254 verifier using the EVM precompiles (ecAdd 0x06, ecMul 0x07, ecPairing 0x08).
// This is the Ethereum counterpart of the project's Solana alt_bn128 verifier: the exact same BN254
// pairing check, e(-A,B)*e(alpha,beta)*e(vk_x,gamma)*e(C,delta) == 1, over the same EIP-197 encoding.
// The proof and verifying key are produced by the Rust prover (arkworks) and passed in.
library Verifier {
    // BN254 base-field modulus (for negating A in G1).
    uint256 constant Q =
        21888242871839275222246405745257275088696311157297823662689037894645226208583;

    // G2 points use EIP-197 order: X = [x_imaginary(c1), x_real(c0)], Y = [y_c1, y_c0].
    struct VK {
        uint256[2] alpha; // G1
        uint256[2][2] beta; // G2
        uint256[2][2] gamma; // G2
        uint256[2][2] delta; // G2
        uint256[2][] ic; // G1[], length = nPublic + 1
    }

    function verify(
        uint256[2] memory a, // proof.A (G1)
        uint256[2][2] memory b, // proof.B (G2)
        uint256[2] memory c, // proof.C (G1)
        uint256[] memory input, // public inputs
        VK memory vk
    ) internal view returns (bool) {
        require(input.length + 1 == vk.ic.length, "bad input length");

        // vk_x = IC[0] + sum(input[i] * IC[i+1])
        uint256[2] memory vkX = vk.ic[0];
        for (uint256 i = 0; i < input.length; i++) {
            vkX = pointAdd(vkX, scalarMul(vk.ic[i + 1], input[i]));
        }

        // -A in G1
        uint256[2] memory negA = [a[0], a[1] == 0 ? 0 : Q - a[1]];

        // Assemble the 4-pair pairing input (each pair = G1 (2 words) + G2 (4 words)).
        uint256[24] memory p;
        (p[0], p[1]) = (negA[0], negA[1]);
        (p[2], p[3], p[4], p[5]) = (b[0][0], b[0][1], b[1][0], b[1][1]);
        (p[6], p[7]) = (vk.alpha[0], vk.alpha[1]);
        (p[8], p[9], p[10], p[11]) = (vk.beta[0][0], vk.beta[0][1], vk.beta[1][0], vk.beta[1][1]);
        (p[12], p[13]) = (vkX[0], vkX[1]);
        (p[14], p[15], p[16], p[17]) = (vk.gamma[0][0], vk.gamma[0][1], vk.gamma[1][0], vk.gamma[1][1]);
        (p[18], p[19]) = (c[0], c[1]);
        (p[20], p[21], p[22], p[23]) = (vk.delta[0][0], vk.delta[0][1], vk.delta[1][0], vk.delta[1][1]);

        uint256[1] memory out;
        bool ok;
        assembly {
            ok := staticcall(gas(), 0x08, p, 0x300, out, 0x20)
        }
        require(ok, "pairing precompile failed");
        return out[0] == 1;
    }

    function scalarMul(uint256[2] memory point, uint256 s) internal view returns (uint256[2] memory r) {
        uint256[3] memory inp = [point[0], point[1], s];
        bool ok;
        assembly {
            ok := staticcall(gas(), 0x07, inp, 0x60, r, 0x40)
        }
        require(ok, "ecMul failed");
    }

    function pointAdd(uint256[2] memory p1, uint256[2] memory p2) internal view returns (uint256[2] memory r) {
        uint256[4] memory inp = [p1[0], p1[1], p2[0], p2[1]];
        bool ok;
        assembly {
            ok := staticcall(gas(), 0x06, inp, 0x80, r, 0x40)
        }
        require(ok, "ecAdd failed");
    }
}
