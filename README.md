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

## 技术路线

- 使用[k256](https://docs.rs/k256/latest/k256/)以及[sha3](https://docs.rs/sha3/latest/sha3/)生成钱包（公私钥对）
- 尝试修改[rust-otp](https://github.com/TimDumol/rust-otp)库进行OTP的生成
