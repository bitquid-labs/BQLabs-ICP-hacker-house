# BQ Labs - Bitcoin Risk Management Protocol (ICP Canisters)

## Overview

BQ Labs introduces an innovative, decentralized risk management layer for the Bitcoin ecosystem built on the Internet Computer Protocol (ICP). This protocol brings secure, transparent insurance solutions to the BTCFi ecosystem, allowing users to underwrite risk, purchase coverage, and efficiently process claims through decentralized governance. The BQ Protocol is specifically designed for ICP canisters, which offer high scalability and seamless user interactions within the ICP ecosystem.

## Architectural Diagram

![ICP Canisters Architectural Diagram](https://brown-high-badger-463.mypinata.cloud/ipfs/QmYB9nPsfP1PEtz71L117zXwpAughVSAi48BZcHWiKuVGW)

## System Architecture

### Key Components

The BQ Protocol is structured to facilitate secure, decentralized risk management by categorizing users into key roles and deploying an architecture comprising several canisters with specific functionalities:

- **Proposers (Cover Buyers):**  
   Users looking to safeguard their Bitcoin activities can use the protocol to buy tailored insurance coverage. After connecting with a non-custodial wallet, proposers can select from various coverage options. The claims process is governed by a decentralized model, involving token holders who assess and vote on proposals.

- **Stakers (Liquidity Providers):**  
   Liquidity providers stake assets in risk pools, earning rewards based on the protocol’s yield framework. This capital underpins the protocol's ability to respond to claims, creating a secure, resilient insurance layer for the BTCFi space.

- **Governance Participants (BQ Token Holders):**  
   Governance participants stake their BQ Tokens to vote on proposals, claims, and key decisions. Their active participation is incentivized through rewards, which foster a knowledgeable and engaged community for effective decentralized decision-making.

### Canister Structure

- **bqBTC Canister:**  
   Manages the minting, burning, and transferring of the BQ token, providing a standardized asset across the protocol for use in cover purchases, staking, and claim payouts.

- **Governance Canister:**  
   Handles governance processes, including proposal creation, voting, and result execution. BQ Token holders leverage this canister to make informed decisions on claims, which maintains the integrity and decentralization of the protocol.

- **Insurance Cover Canister:**  
   Facilitates the cover creation, purchasing, and tracking of user covers. This canister integrates with the governance canister for claim assessments and the bqBTC canister for payments.

- **Insurance Pool Canister:**  
   Manages insurance pools where users can stake their assets. This canister tracks each pool’s liquidity, deploys capital during adverse events, and rewards liquidity providers based on their contributions.

## Core Features

1. **Purchase Cover:**  
   Users can browse available cover options, select desired coverage periods, and complete purchases. The Insurance Cover Canister verifies eligibility, facilitates transactions, and securely stores cover information.

2. **Staking:**  
   Users can stake assets in designated pools to earn yield, with real-time updates on their staking balances and accrued rewards. This ensures continuous liquidity in the protocol to support risk underwriting.

3. **Claim Assessment Module:**  
   Users can file claims on their covers. Governance participants then assess the claim’s validity via the Governance Canister, and approved claims are paid out from the pool’s liquidity.

4. **Decentralized Governance:**  
   BQ Token holders manage protocol decisions through a governance process that includes proposal submission, voting, and result execution. Voters earn additional rewards for active participation, maintaining a secure and community-driven decision-making process.

5. **Dynamic Pricing Algorithms:**  
   The protocol continuously recalculates coverage prices, claim limits, and pool utilization rates using real-time risk data, ensuring that coverage remains fairly priced and aligned with actual risk.

## Technical Details

- **Canister Architecture:** Developed using Rust with candid interfaces, each canister is dedicated to a specific function within the protocol.
- **Governance Mechanism:** Integrated directly into the protocol, governance is executed entirely on-chain, ensuring transparency and security.
- **Token Integration:** Utilizes bqBTC as a unified currency across all operations.
- **Storage and Data Security:** Leveraging ICP’s distributed ledger and candid, the protocol ensures that all transactions, data, and assets are secure and immutable.

## Deployed Canisters
1. **BQBTC Canister** : https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=ehul3-6aaaa-aaaan-qzouq-cai
2. **Governance Canister** : https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=ejwgt-fqaaa-aaaan-qzovq-cai
3. **Cover Canister** : https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=eoxah-iiaaa-aaaan-qzova-cai
4. **Pool Canister** : https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=e4rx6-eyaaa-aaaan-qzowa-cai

## Getting Started

1. **Install Dependencies:**  
   - Ensure you have the Internet Computer SDK and Rust installed.
   - Compile each canister with: `cargo build --release --target wasm32-unknown-unknown`.

2. **Generate Candid Files:**  
   Use the command `candid-extractor target/wasm32-unknown-unknown/release/<CANISTER>.wasm <CANISTER>.did` for each canister to generate candid files.

3. **Deploy Canisters:**  
   Deploy each canister using the ICP SDK, linking them together through cross-canister calls as needed.

4. **Connect with Frontend:**  
   Implement frontend integration using Dfinity’s agent library to allow user interactions with the deployed canisters.

## Conclusion

BQ Labs’ Bitcoin Risk Management Protocol on ICP delivers a decentralized and resilient risk management solution tailored to the needs of the BTCFi ecosystem. By addressing challenges in on-chain trust, transparency, and claim processing, BQ Labs empowers users with an efficient and community-driven insurance platform. With its robust canister architecture, real-time governance model, and dynamic risk assessment, BQ Labs is poised to enhance security and trust across Bitcoin financial operations.
