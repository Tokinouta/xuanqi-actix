use mongodb::{
    options::{ClientOptions, Credential, ServerAddress},
    Client, Database,
};

pub async fn create_client() -> Client {
    // 尽管这里面看上去都是同步API，但是实际还是需要一个异步环境来执行。否则会报错。
    // 千万注意这点⬆
    // 以及async是可以没有await的😂
    let credential = Credential::builder()
        .username(Some("mongoadmin".to_string()))
        .password(Some("secret".to_string()))
        .build();
    // Parse a connection string into an options struct.
    let client_options = ClientOptions::builder()
        .hosts(vec![ServerAddress::parse("localhost:27017").expect("msg")])
        .app_name(Some("My App".to_string()))
        .credential(credential)
        .build();

    // Get a handle to the deployment.
    let client = Client::with_options(client_options).expect("failed to connect");
    client
}

pub async fn list_database_names(client: &Client) -> Vec<String> {
    // List the names of the databases in that deployment.
    let names = client
        .list_database_names(None, None)
        .await
        .expect("failed to list");

    for db_name in names.iter() {
        println!("{}", db_name);
    }

    names
}

pub async fn create_database(client: &Client, name: &str) -> Option<Database> {
    // 这里实际只创建了一个条目，并没有真正写入mongodb
    // 需要往里面写入一些document才能让他实际出现在数据库中。
    let databases = list_database_names(client).await;
    if !databases.contains(&name.to_string()) {
        println!("ready to create");
        Some(client.database(name))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use mongodb::bson::{doc, Document};

    // 注意这个惯用法：在 tests 模块中，从外部作用域导入所有名字。
    // 注意私有的函数也可以被测试！
    use super::*;

    #[test]
    fn test_mongodb() {
        // 这里注意client和使用它的函数需要在同一个运行环境里，不能由两个block_on函数分别执行
        // 否则第二个block_on可能获取不到第一个的一些信息，导致报错“Server selection timeout: No available servers.”。
        let a = || async {
            let client = create_client().await;
            list_database_names(&client).await
        };
        let res = tokio_test::block_on(a());
        assert_eq!(res, vec!["admin", "config", "local"])
    }

    #[test]
    fn test_create_database() {
        let name = "rarara";
        let a = || async {
            let client = create_client().await;
            let db = create_database(&client, name).await.unwrap();

            // 创建一个collection用于存储数据
            let collection = db.collection::<Document>("books");
            // 待写入的数据
            let docs = vec![
                doc! { "title": "1984", "author": "George Orwell" },
                doc! { "title": "Animal Farm", "author": "George Orwell" },
                doc! { "title": "The Great Gatsby", "author": "F. Scott Fitzgerald" },
            ];

            // Insert some documents into the "rarara.books" collection.
            // 写入完成之后才真正能够在数据库中获取到rarara库
            collection.insert_many(docs, None).await.expect("msg");
            list_database_names(&client).await
        };
        let res = tokio_test::block_on(a());
        assert!(res.contains(&name.to_string()));
        let b = || async {
            let client = create_client().await;
            client
                .database(name)
                .drop(None) // 删除rarara，毕竟是一个测试用的库
                .await
                .expect("no such database");
            list_database_names(&client).await
        };
        let res = tokio_test::block_on(b());
        assert!(!res.contains(&name.to_string()));
    }
}
