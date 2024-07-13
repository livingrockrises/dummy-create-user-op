use alloy_core::{
    hex,
    primitives::{keccak256, Uint, U256},
};
use alloy_sol_types::{sol, SolValue};

sol!(
    #[derive(Debug, PartialEq)]
    struct UserOperation {
        address sender;
        uint256 nonce;
        bytes32 initCode;
        bytes32 callData;
        uint256 callGasLimit;
        uint256 verificationGasLimit;
        uint256 preVerificationGas;
        uint256 maxFeePerGas;
        uint256 maxPriorityFeePerGas;
        bytes32 paymasterAndData;
    }

    // Review bytes32 vs bytes
    struct NewUserOperation {
        address sender;
        uint256 nonce;
        address factory; // optionally undefined
        bytes32 factoryData; // optionally undefined
        bytes32 callData;
        uint256 callGasLimit;
        uint256 verificationGasLimit;
        uint256 preVerificationGas;
        uint256 maxFeePerGas;
        uint256 maxPriorityFeePerGas;
        address paymaster; // optionally undefined
        uint256 paymasterVerificationGasLimit; // optionally undefined
        uint256 paymasterPostOpGasLimit; // optionally undefined
        bytes32 paymasterData; // optionally undefined
        // signature ?
    }

    struct PackedUserOperation {
        address sender;
        uint256 nonce;
        bytes initCode;
        bytes callData;
        bytes32 accountGasLimits;
        uint256 preVerificationGas;
        bytes32 gasFees;
        bytes paymasterAndData;
        bytes signature;
    }
);

// need another function to pack user operation.

fn main() {
    let entry_point_version = 6;
    let entry_point_address_v6 = "0x5ff137d4b0fdcd49dca30c7cf57e578a026d2789";
    let entry_point_address_v7 = "0x0000000071727De22E5E9d8BAf0edAc6f37da032";
    let chain_id = 80002;

    let user_op_v6 = UserOperation {
        sender: "0xe6dBb5C8696d2E0f90B875cbb6ef26E3bBa575AC".parse().unwrap(),
        nonce: U256::from(1617),
        initCode: keccak256(hex::decode("0x").unwrap()),
        callData: keccak256(hex::decode("0x0000189a0000000000000000000000003079b249dfde4692d7844aa261f8cf7d927a0da5000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000").unwrap()),
        callGasLimit: Uint::<256, 4>::from(14177),
        verificationGasLimit: Uint::<256, 4>::from(54701),
        preVerificationGas: Uint::<256, 4>::from(59393),
        maxFeePerGas: U256::from(18000000000u64),
        maxPriorityFeePerGas: U256::from(17999999985u64),
        paymasterAndData: keccak256(hex::decode("0x").unwrap()),
    };

    // let user_op_v7 = //

    if entry_point_version == 6 {
        let user_op_hash = keccak256(&user_op_v6.abi_encode());

        // pack (userOpHash, entryPointAddress, chainId)
        let ep_address_padded = format!("{:0>64}", entry_point_address_v6.trim_start_matches("0x"));
        let chain_id_hex = format!("{:x}", chain_id);
        let chain_id_padded = format!("{:0>64}", chain_id_hex);
        let concatenated = format!("{}{}{}", user_op_hash, ep_address_padded, chain_id_padded);
        let concatenated_bytes = hex::decode(concatenated).expect("Decoding hex to byte failed");
        let hash = keccak256(&concatenated_bytes);

        println!("Hash: {:?}", hash);
    } else if entry_point_version == 7 {
        // let user_op_hash = keccak256(&user_op_v6.abi_encode());
        // let ep_address_padded = format!("{:0>64}", entry_point_address_v6.trim_start_matches("0x"));
        // let chain_id_hex = format!("{:x}", chain_id);
        // let chain_id_padded = format!("{:0>64}", chain_id_hex);
        // let concatenated = format!("{}{}{}", user_op_hash, ep_address_padded, chain_id_padded);
        // let concatenated_bytes = hex::decode(concatenated).expect("Decoding hex to byte failed");
        // let hash = keccak256(&concatenated_bytes);

        // println!("Hash: {:?}", hash);
    } else {
        panic!("Invalid entry point version");
    }
}
