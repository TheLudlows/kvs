# chat-demo
a rusty memory cache for alibaba cloud tianchi competition.
程序需要监听8080端口，特别需要注意的是暴露init接口，代表是否加载完成数据，加载完成返回httpstatuscode为200，并且内容是 ‘ok’ 这个字符串

it contains:
* source code 
* 用户需要提交的一个压缩文件deploy_application.zip，可以参考build目录里面的deploy_application.zip，它里面只有一个ros文件，注意ros文件里面需要修改的就是我们的镜像id，这个镜像里面包含我们的可执行程序，直接使用userdata进行启动我们的程序。ros里面有一个nas url参数，它程序启动的时候需要读取的数据所在，注意我们不要修改nas目录中的文件内容，否则会被裁定违规，一种方法就是我们在启动的时候直接将nas里里面的内容拷贝到我们的/data目录，然后对data目录进行操作，这样就不会改动nas里面的内容了