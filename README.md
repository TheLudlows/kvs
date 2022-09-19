#### 基本思路
使用基于Tokio的warp作为Web服务器，KV和ZSet存储采用了线程安全的`Dashmap`。 ZSet内部采用双map来映射score和value。

1.tokio worker线程数
2.jemallocator分配内存
3.fast hash function create (not security)
4.http pipeline
5.使用Bytes存储KV字符串减少Copy，效果一般

