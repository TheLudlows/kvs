#### 基本思路
使用基于Tokio的warp作为Web服务器，KV和ZSet存储采用了线程安全的`Dashmap`。 ZSet内部采用双map来映射score和value。

1.tokio worker线程数
2.jemallocator分配内存
3.fast hash function create (not security)
4.http pipeline
5.使用Bytes存储KV字符串减少Copy，效果一般

#### 复赛思路

在初赛的基础上构建基于类似redis-cluster模型,三个机器分别持有一段token

1. 每个节点只负责在内存中保存自己的数据，当数据超过阈值，则会淘汰掉最久没有使用的数据
2. 每个节点会将数据转发到对应的节点上去处理
3. 综合读写，没有做局部缓存
