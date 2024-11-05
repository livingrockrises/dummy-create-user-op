use alloy_core::{
    hex,
    primitives::{keccak256, FixedBytes, Uint, Address, U256},
};
use alloy_sol_types::{sol, SolValue};

extern crate primitive_types;
use primitive_types::{H160, H256};

use std::str::FromStr;

// Define Solidity structs using sol! macro
sol!(
    #[derive(Debug, PartialEq)]
    // This is intermediate object ready for ABI encoding after hashing dynamic bytes.
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

    #[derive(Debug, PartialEq)]
    // This is intermediate object ready for ABI encoding after hashing dynamic bytes.
    struct PackedUserOperation {
        address sender;
        uint256 nonce;
        bytes32 initCode; // hashed
        bytes32 callData; // hashed
        bytes32 accountGasLimits; // 2 16 bytes
        uint256 preVerificationGas; 
        bytes32 gasFees; // 2 16 bytes
        bytes32 paymasterAndData; // hashed
    }
);

#[derive(Debug, PartialEq)]
struct UserOperationInput {
    sender: H160,
    nonce: U256,
    initCode: H256,
    callData: H256,
    callGasLimit: U256,
    verificationGasLimit: U256,
    preVerificationGas: U256,
    maxFeePerGas: U256,
    maxPriorityFeePerGas: U256,
    paymasterAndData: Option<H256>,  // Now optional, defaults to None which can represent 0x0
}

#[derive(Debug, PartialEq)]
struct NewUserOperationInput {
    sender: H160,
    nonce: U256,
    factory: Option<H160>, // Optional Ethereum address
    factoryData: Option<Vec<u8>>, // Optional dynamic bytes
    callData: Vec<u8>, // Dynamic bytes
    callGasLimit: U256,
    verificationGasLimit: U256,
    preVerificationGas: U256,
    maxFeePerGas: U256,
    maxPriorityFeePerGas: U256,
    paymaster: Option<H160>, // Optional Ethereum address
    paymasterVerificationGasLimit: Option<U256>, // Optional uint256
    paymasterPostOpGasLimit: Option<U256>, // Optional uint256
    paymasterData: Option<Vec<u8>>, // Optional dynamic bytes
}

impl Default for NewUserOperationInput {
    fn default() -> Self {
        NewUserOperationInput {
            sender: H160::default(),
            nonce: U256::from(0),
            factory: None,
            factoryData: None,
            callData: Vec::new(),
            callGasLimit: U256::from(0),
            verificationGasLimit: U256::from(0),
            preVerificationGas: U256::from(0),
            maxFeePerGas: U256::from(0),
            maxPriorityFeePerGas: U256::from(0),
            paymaster: None,
            paymasterVerificationGasLimit: None,
            paymasterPostOpGasLimit: None,
            paymasterData: None,
        }
    }
}

fn pack_user_operation(user_op: &NewUserOperationInput) -> PackedUserOperation {
    let hashed_init_code;
    if let (Some(factory), Some(factory_data)) = (&user_op.factory, &user_op.factoryData) {
        // Convert the fixed-size byte array to a slice and concatenate with the factory data slice.
        let factory_bytes = factory.to_fixed_bytes();
        let factory_bytes_slice = &factory_bytes[..]; // Convert [u8; 20] to &[u8]
        let factory_data_bytes = factory_data.as_slice(); // Already &[u8]
    
        // Concatenate into a single Vec<u8>
        let mut data = Vec::new();
        data.extend_from_slice(factory_bytes_slice);
        data.extend_from_slice(factory_data_bytes);
    
        // Pass the concatenated data to keccak256
        hashed_init_code = keccak256(&data);
    } else {
        hashed_init_code = keccak256(&vec![]);
    }

    let hashed_call_data = keccak256(&user_op.callData);

    let hashed_paymaster_and_data;
    if let (Some(paymaster), Some(verification_gas), Some(post_gas), Some(pay_data)) = 
    (&user_op.paymaster, &user_op.paymasterVerificationGasLimit, &user_op.paymasterPostOpGasLimit, &user_op.paymasterData) {
        // Create a new Vec<u8> to hold all concatenated bytes
        let mut data = Vec::new();

        // Extend the Vec with the paymaster address bytes
        data.extend_from_slice(&paymaster.to_fixed_bytes());

        // Extend the Vec with the relevant bytes from verification gas limit
        // Assuming you want to skip the first 16 bytes and take the rest, make sure the indexing is correct
        let verification_gas_bytes: [u8; 32] = verification_gas.to_be_bytes();
        data.extend_from_slice(&verification_gas_bytes[16..]); // Adjust according to the actual data size needs

        // Similar handling for post gas limit
        let post_gas_bytes: [u8; 32] = post_gas.to_be_bytes();
        data.extend_from_slice(&post_gas_bytes[16..]); // Adjust according to the actual data size needs

        // Extend the Vec with the dynamic paymaster data
        data.extend_from_slice(pay_data);

        // Compute the keccak256 hash of the concatenated data
        hashed_paymaster_and_data = keccak256(&data)
    } else {
        hashed_paymaster_and_data = keccak256(&vec![])
    };

    let verification_gas_bytes: [u8; 32] = user_op.verificationGasLimit.to_be_bytes();
    let call_gas_bytes: [u8; 32] = user_op.callGasLimit.to_be_bytes();

    let account_gas_limits =
        [   &call_gas_bytes[16..],
            &verification_gas_bytes[16..]
        ].concat()
    ;

    let max_fee_per_gas_bytes: [u8; 32] = user_op.maxFeePerGas.to_be_bytes();
    let max_priority_fee_per_gas_bytes: [u8; 32] = user_op.maxPriorityFeePerGas.to_be_bytes();

    let gas_fees = 
        [
            &max_priority_fee_per_gas_bytes[16..],
            &max_fee_per_gas_bytes[16..]
        ].concat()
    ;

    let sender_address: FixedBytes<20> = FixedBytes::from(user_op.sender.0);
    let init_code_fixed: FixedBytes<32> = FixedBytes::from(hashed_init_code.0); 


    PackedUserOperation {
        sender: Address(sender_address),
        nonce: user_op.nonce,
        initCode: init_code_fixed,
        callData: FixedBytes::from(hashed_call_data.0),
        accountGasLimits:  FixedBytes::from_slice(account_gas_limits.as_slice()),
        preVerificationGas: user_op.preVerificationGas,
        gasFees:  FixedBytes::from_slice(gas_fees.as_slice()),
        paymasterAndData: FixedBytes::from(hashed_paymaster_and_data.0),
    }
}


fn main() {
    let entry_point_version = 7;
    let entry_point_address_v6 = "0x5ff137d4b0fdcd49dca30c7cf57e578a026d2789";
    let entry_point_address_v7 = "0x0000000071727De22E5E9d8BAf0edAc6f37da032";
    let chain_id = 31337; // 31337 for comparing against hardhat

    let user_op_v6 = UserOperation {
        sender: "0xe6dBb5C8696d2E0f90B875cbb6ef26E3bBa575AC".parse().unwrap(),
        nonce: U256::from(1617),
        initCode: keccak256(&vec![]),
        callData: keccak256(&hex::decode("0x0000189a0000000000000000000000003079b249dfde4692d7844aa261f8cf7d927a0da5000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000").unwrap()),
        callGasLimit: Uint::<256, 4>::from(14177),
        verificationGasLimit: Uint::<256, 4>::from(54701),
        preVerificationGas: Uint::<256, 4>::from(59393),
        maxFeePerGas: U256::from(18000000000u64),
        maxPriorityFeePerGas: U256::from(17999999985u64),
        paymasterAndData: keccak256(&vec![]),
    };

    let user_op_v7_input = NewUserOperationInput {
        sender: "0xc10035C6c74e8Af054897Ff6092Dc3eC49e2eFc6".parse().unwrap(),
        nonce: U256::from_str("1083597386547022464258429625247069249537518245239347114964906802352750592").unwrap(),
        callData: hex::decode("0xe9ae5c53000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000078ba45f9ef3e4fb871113f68217604af3626de8c440000000000000000000000000000000000000000000000000000000000000000a9059cbb0000000000000000000000003c44cdddb6a900fa2b585dd299e03d12fa4293bc0000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000").unwrap(),
        callGasLimit: Uint::<256, 4>::from(1500000),
        verificationGasLimit: Uint::<256, 4>::from(1500000),
        preVerificationGas: Uint::<256, 4>::from(2000000),
        maxFeePerGas: U256::from(20000000000u64),
        maxPriorityFeePerGas: U256::from(10000000000u64),
        ..Default::default() // Fill all other fields with default values
    };

    if entry_point_version == 6 {
        let user_op_hash = keccak256(&user_op_v6.abi_encode());

        let ep_address_padded = format!("{:0>64}", entry_point_address_v6.trim_start_matches("0x"));
        let chain_id_hex = format!("{:x}", chain_id);
        let chain_id_padded = format!("{:0>64}", chain_id_hex);
        let concatenated = format!("{}{}{}", hex::encode(user_op_hash), ep_address_padded, chain_id_padded);
        let concatenated_bytes = hex::decode(concatenated).expect("Decoding hex to byte failed");
        let hash = keccak256(&concatenated_bytes);

        println!("Hash: {:?}", hash);
    } else if entry_point_version == 7 {
        let user_op_v7 = pack_user_operation(&user_op_v7_input);
        // println!("Packed UserOperation V7 ABI encode: {:?}", hex::encode(user_op_v7.abi_encode()));
        let user_op_v7_hash = keccak256(&user_op_v7.abi_encode());

        let ep_address_padded = format!("{:0>64}", entry_point_address_v7.trim_start_matches("0x"));
        let chain_id_hex = format!("{:x}", chain_id);
        let chain_id_padded = format!("{:0>64}", chain_id_hex);
        let concatenated = format!("{}{}{}", hex::encode(user_op_v7_hash), ep_address_padded, chain_id_padded);
        let concatenated_bytes = hex::decode(concatenated).expect("Decoding hex to byte failed");
        let hash = keccak256(&concatenated_bytes);

        println!("Hash: {:?}", hash);
    } else {
        panic!("Invalid entry point version");
    }
}



