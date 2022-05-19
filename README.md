# 嵌入式作业 - ETH硬件钱包

## 实现功能

- [ ] ETH钱包生成
  - [ ] k256 algorithm
  - [ ] 对测信道攻击的防护（可选）
- [ ] 基于用户口令的保护
  - [ ] 在小键盘输入数字密码解锁后才能使用其他功能
- [ ] 基于OTP的保护（可选）
  - [ ] 通过小屏幕显示OTP导入二维码（可选）
  - [ ] 敏感操作需要OTP验证（如修改口令）
- [ ] 基于brownout detection的断电保护

## 技术路线

- 使用[k256](https://docs.rs/k256/latest/k256/)以及[sha3](https://docs.rs/sha3/latest/sha3/)生成钱包（公私钥对）
- 尝试修改[rust-otp](https://github.com/TimDumol/rust-otp)库进行OTP的生成
- 在内存中初始化一块safe zone，将钱包公私钥以及OTP配置放入其中，其结构如下：
  
  ```
  -------------
  MAGIC HEADER
  -------------
  KEY-PAIRS
  -------------
  OTP-CONFIG
  -------------
  ```

  该区域内数据全部使用对称密钥加密。用户初次使用钱包需要设置口令pass，接下来程序生成钱包和OTP配置，使用`keccak256(pass)`作为ChaCha20加密的密钥将其加密后放入safe zone。同时我们需要加密一个MAGIC NUMBER放入safe zone。该区域内的数据不使用时都处于加密状态，使用时

  1. 要求用户输入口令
  2. 使用口令解密MAGIC NUMBER，验证是否正确
  3. 若正确，说明口令正确，使用它解密待使用的数据即可
