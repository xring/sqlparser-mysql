extern crate sqlparser_mysql;

use sqlparser_mysql::dds::{AlterTableStatement, CreateTableStatement};

#[test]
fn parse_create_table() {
    let create_sqls = vec![
        r###"CREATE TABLE `process_type` (`last_update_tm` timestamp(0))"###,
        r###"CREATE TABLE `process_type` (`last_update_tm` timestamp(0) NOT NULL DEFAULT CURRENT_TIMESTAMP(0) ON UPDATE CURRENT_TIMESTAMP(0))"###,
        "CREATE TABLE foo.order_items (order_id INT, product_id INT, quantity INT, PRIMARY KEY(order_id, product_id), FOREIGN KEY (product_id) REFERENCES product (id))",
        "CREATE TABLE employee (id INT, name VARCHAR(100), department_id INT, PRIMARY KEY(id), FOREIGN KEY (department_id) REFERENCES department(id))",
        "CREATE TABLE my_table (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(100), age INT)",
        "CREATE TEMPORARY TABLE temp_table (id INT, score DECIMAL(5, 2))",
        "CREATE TABLE IF NOT EXISTS my_table (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(100), age INT)",
        "CREATE TABLE department (id INT AUTO_INCREMENT, name VARCHAR(100), PRIMARY KEY(id))",
        "CREATE TABLE product (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(100), price DECIMAL(10,2), category_id INT, INDEX(category_id))",
        "CREATE TABLE my_table_copy LIKE my_table",
        "CREATE TABLE IF NOT EXISTS my_table_copy LIKE my_table",
        "CREATE TEMPORARY TABLE temp_table_copy LIKE temp_table;",
        "CREATE TABLE department_copy LIKE department",
        "CREATE TEMPORARY TABLE IF NOT EXISTS temp_table_copy LIKE my_table",
        "CREATE TABLE IF NOT EXISTS bar.employee_archives LIKE foo.employee",
        "CREATE TABLE product (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(100), price DECIMAL(10,2), category_id INT, INDEX(category_id))",
        "CREATE TABLE my_table_filtered AS SELECT * FROM my_table WHERE age < 30;",
        "CREATE TABLE employee_dept_10 AS SELECT * FROM employee WHERE department_id = 10",
        "CREATE TEMPORARY TABLE IF NOT EXISTS temp_dept_20 AS SELECT * FROM department WHERE id = 20",
        "CREATE TABLE active_products AS SELECT * FROM product WHERE price > 0",
        "CREATE TABLE sales_by_product AS SELECT product_id, SUM(quantity) AS total_sales FROM order_items GROUP BY product_id",
        "CREATE TEMPORARY TABLE IF NOT EXISTS temp_order_summary AS SELECT order_id, SUM(quantity) AS total_items FROM order_items GROUP BY order_id",
        "CREATE TABLE employee_names AS SELECT name FROM employee",
        "CREATE TABLE new_table AS SELECT name, age FROM my_table WHERE age > 18",
        "CREATE TABLE unique_names IGNORE AS SELECT DISTINCT name FROM my_table",
        "CREATE TABLE employee_summary AS SELECT department_id, COUNT(*) AS employee_count FROM employee GROUP BY department_id",
        "CREATE TEMPORARY TABLE temp_employee AS SELECT * FROM employee WHERE department_id = 3",
        "CREATE TABLE IF NOT EXISTS employee_backup AS SELECT * FROM employee",
        "CREATE TABLE product_prices AS SELECT name, price FROM product WHERE price BETWEEN 10 AND 100",
        "CREATE TABLE author ( a_id int not null, a_fname varchar(20), a_lname varchar(20), a_mname varchar(20), a_dob date, a_bio int, PRIMARY KEY(a_id))",
        "CREATE TABLE customer ( c_id int not null, c_uname varchar(20), c_passwd varchar(20), c_fname varchar(17), c_lname varchar(17), c_addr_id int, c_phone varchar(18), c_email varchar(50), c_since date, c_last_login date, c_login timestamp, c_expiration timestamp, c_discount real, c_balance double, c_ytd_pmt double, c_birthdate date, c_data int, PRIMARY KEY(c_id))",
        "CREATE TABLE item ( i_id int not null, i_title varchar(60), i_a_id int, i_pub_date date, i_publisher varchar(60), i_subject varchar(60), i_desc text, i_related1 int, i_related2 int, i_related3 int, i_related4 int, i_related5 int, i_thumbnail varchar(40), i_image varchar(40), i_srp double, i_cost double, i_avail date, i_stock int, i_isbn char(13), i_page int, i_backing varchar(15), i_dimensions varchar(25), PRIMARY KEY(i_id))",
        "CREATE TABLE user (user_id int(5) unsigned NOT NULL auto_increment,user_name varchar(255) binary NOT NULL default '',user_rights tinyblob NOT NULL default '',user_password tinyblob NOT NULL default '',user_newpassword tinyblob NOT NULL default '',user_email tinytext NOT NULL default '',user_options blob NOT NULL default '',user_touched char(14) binary NOT NULL default '',UNIQUE KEY user_id (user_id)) ENGINE=MyISAM PACK_KEYS=1;",
        "CREATE TABLE `admin_assert` (`assert_id` int(10) unsigned NOT NULL Auto_Increment COMMENT 'Assert ID',`assert_type` varchar(20) DEFAULT NULL COMMENT 'Assert Type',`assert_data` text COMMENT 'Assert Data',PRIMARY KEY (`assert_id`)) ENGINE=InnoDB DEFAULT CHARSET=utf8;",
        "CREATE TABLE `postcode_city` (`id` int(10) unsigned NOT NULL Auto_Increment COMMENT 'Id',`country_code` varchar(5) NOT NULL COMMENT 'Country Code',`postcode` varchar(20) NOT NULL COMMENT 'Postcode',`city` text NOT NULL COMMENT 'City',PRIMARY KEY (`id`)) ENGINE=InnoDB Auto_Increment=52142 DEFAULT CHARSET=utf8 COMMENT='Postcode -> City';",
    ];
    for i in 0..create_sqls.len() {
        println!("{}/{}", i + 1, create_sqls.len());
        let res = CreateTableStatement::parse(create_sqls[i]);
        println!("{:?}", res);
        assert!(res.is_ok());
    }
}

#[test]
fn parse_alter_table() {
    let alter_sqls = vec![
        r###"ALTER TABLE common_stats.event_event_attr_link ADD COLUMN filter   TINYINT(4) DEFAULT 0 COMMENT '统计过滤(1=启用过滤；0=禁用过滤)', ADD COLUMN filter_name VARCHAR(64) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '统计筛选字段名称';"###,
        "ALTER TABLE tbl_order DISABLE KEYS",
        "ALTER TABLE tbl_order ORDER BY col_3",
        "ALTER TABLE tbl_customer ENABLE KEYS",
        "ALTER TABLE tbl_order DROP COLUMN col_6",
        "ALTER TABLE tbl_order RENAME TO tbl_customer_31",
        "ALTER TABLE tbl_order ADD INDEX idx_34 (col_14)",
        "ALTER TABLE tbl_customer ADD COLUMN col_74 VARCHAR(255)",
        "ALTER TABLE tbl_customer RENAME COLUMN col_20 TO col_30",
        "ALTER TABLE tbl_product CHANGE COLUMN col_1 col_21 DATE",
        "ALTER TABLE tbl_inventory ADD CONSTRAINT UNIQUE (col_19)",
        "ALTER TABLE tbl_order ADD FULLTEXT INDEX ft_idx_87 (col_1)",
        "ALTER TABLE tbl_product CONVERT TO CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci",
        "ALTER TABLE test_table ADD COLUMN new_column INT;",
        "ALTER TABLE test_table ADD COLUMN another_column VARCHAR(255) AFTER new_column;",
        "ALTER TABLE test_table ADD INDEX (new_column);",
        "ALTER TABLE test_table ADD FULLTEXT INDEX (another_column);",
        "ALTER TABLE test_table ADD SPATIAL INDEX (another_column);",
        "ALTER TABLE test_table ADD CONSTRAINT fk_example FOREIGN KEY (new_column) REFERENCES other_table(other_column);",
        "ALTER TABLE test_table ADD CONSTRAINT chk_column CHECK (new_column > 0) NOT ENFORCED;",
        "ALTER TABLE test_table DROP CHECK chk_column;",
        "ALTER TABLE test_table ALTER CHECK chk_column NOT ENFORCED;",
        "ALTER TABLE test_table ENGINE = InnoDB;",
        "ALTER TABLE test_table MODIFY COLUMN new_column BIGINT NOT NULL;",
        "ALTER TABLE test_table ALTER COLUMN new_column SET DEFAULT 10;",
        "ALTER TABLE test_table ALTER COLUMN new_column DROP DEFAULT;",
        "ALTER TABLE test_table MODIFY COLUMN another_column VARCHAR(255) FIRST;",
        "ALTER TABLE test_table RENAME COLUMN another_column TO renamed_column;",
        "ALTER TABLE test_table DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;",
        "ALTER TABLE test_table CONVERT TO CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;",
        "ALTER TABLE test_table DISABLE KEYS;",
        "ALTER TABLE test_table ENABLE KEYS;",
        "ALTER TABLE test_table DISCARD TABLESPACE;",
        "ALTER TABLE test_table IMPORT TABLESPACE;",
        "ALTER TABLE test_table DROP COLUMN renamed_column;",
        "ALTER TABLE test_table DROP INDEX unique_index_name;",
        "ALTER TABLE test_table DROP PRIMARY KEY;",
        "ALTER TABLE test_table DROP FOREIGN KEY fk_example;",
        "ALTER TABLE test_table FORCE;",
        "ALTER TABLE test_table ALTER INDEX index_name VISIBLE;",
        "ALTER TABLE test_table ALTER INDEX index_name INVISIBLE;",
        "ALTER TABLE test_table ADD PRIMARY KEY (new_column);",
        "ALTER TABLE test_table ADD UNIQUE INDEX unique_index_name (another_column);",
        "ALTER TABLE tbl_product ADD COLUMN col_name160 VARCHAR(255) NOT NULL",
        "ALTER TABLE tbl_customer DROP COLUMN col_name91",
        "ALTER TABLE tbl_inventory MODIFY COLUMN col_name73 TEXT",
        "ALTER TABLE tbl_product CHANGE COLUMN col_name28 col_name217 DATETIME",
        "ALTER TABLE tbl_inventory ADD INDEX idx_name145 (col_name51)",
        "ALTER TABLE tbl_order DROP INDEX idx_name23",
        "ALTER TABLE tbl_product RENAME TO tbl_product_new",
        "ALTER TABLE tbl_order ADD PRIMARY KEY (col_name49)",
        "ALTER TABLE tbl_order DROP PRIMARY KEY",
        "ALTER TABLE tbl_customer ADD FOREIGN KEY (col_name74) REFERENCES tbl_order(order_id)",
        "ALTER TABLE tbl_inventory DROP FOREIGN KEY fk_name46",
        "ALTER TABLE demo ADD name VARCHAR(128) NULL DEFAULT NULL AFTER age",
        "ALTER TABLE `process_template_config` ADD template_admin_approver json NULL DEFAULT NULL COMMENT '模板管理员';"
    ];

    for i in 0..alter_sqls.len() {
        println!("{}/{}", i + 1, alter_sqls.len());
        let res = AlterTableStatement::parse(alter_sqls[i]);
        // res.unwrap();
        // println!("{:?}", res);
        println!("{:?}", res);
        assert!(res.is_ok());
    }
}
