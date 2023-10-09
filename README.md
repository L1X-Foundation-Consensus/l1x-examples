# L1X Examples of Contracts

### Content
* **l1x-contract** - hello world contract
* **l1x-cross-contract** - example of L1XVM -> L1XVM cross-contract call. The contract calls `l1x-contract` so `l1x-contract` should be deployed and intialized. After that the l1x-contract's instance address should be used in the cross-contract call code.
* **l1x-evm-cross-contract** example of L1XVM -> L1XEVM cross-contract call. The contract calls Solidity ERC20 contract so the contract should be deployed and intialized first.
* **l1x-ft** - ERC20 token implementaion
* **l1x-nft** - ERC-721 token implementation (implementaion is limited)
* **l1x-transfer-token** - example with native L1X tokens transfer
* **source-registry** - example of a x-talk source registry contract
* **xtalk-nft-ad-flow-contract** - example of a x-talk contract
* **xtalk-swap-flow** - example of a x-talk contract

### How to build
```bash
devbox shell
# Init the development workspace
devbox run init_setup
# compile the contract by the package name
devbox run compile l1x-ft
# OR compile all contracts
devbox run compile_all
# Find *.o files in l1x-artifacts/
```
Feel free to modify `devbox.json` and `l1x-conf/`