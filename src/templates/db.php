<?php
require_once("../config/db.php");

Db::$db = new PDO(
	"mysql:host=" . DB_HOST . ";dbname=" . DB_NAME,
	DB_USER,
	DB_PASSWORD
);

class Db {
	public static PDO $db;
	public static function query(string $query): PDOStatement {
		$result = self::$db->query($query);
		if($result === false) {
			die("Sql query failed: " . self::$db->errorCode());
		}

		return $result;
	}
}
?>
