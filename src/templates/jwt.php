<?php
require_once("../config/jwt.php");

class Jwt {
	public array $payload = [];
	private string $signature = "";

	function import(string $json_string): bool {
		$parts = explode(".", $json_string);

		if(count($parts) !== 3) { return false; }

		$header_base64 = $parts[0];
		$header = json_decode(base64_decode($header_base64), true);
		if($header["alg"] !== "HS512") { return false; }
		if($header["typ"] !== "JWT") { return false; }

		$payload_base64 = $parts[1];
		$this->payload = json_decode(base64_decode($payload_base64), true);

		$this->signature = $parts[2];

		return true;
	}

	function export(): string {
		$this->signature = $this->createSignature();
		$signature_base64 = $this->signature;
	
		$header_json = json_encode([
			"typ" => "JWT",
			"alg" => "HS512"
		]);
		$header_base64 = self::b64_custom_encode($header_json);
		$payload_base64 = self::b64_custom_encode(json_encode($this->payload));

		return "{$header_base64}.{$payload_base64}.{$signature_base64}";
	}
	
	function valid(): bool {
		return strcmp($this->signature, $this->createSignature()) === 0;
	}

	private function updateTime(): void {
		$this->payload += [
			"iat" => time(),
			"exp" => time() + JWT_EXP
		];
		return;
	}

	private function createSignature(): string {
		$header_base64 = base64_encode(json_encode([
			"typ" => "JWT",
			"alg" => "HS512"
		]));
		$header_base64 = self::b64_custom_encode($header_base64);
		$payload_base64 = json_encode($this->payload);
		$payload_base64 = self::b64_custom_encode($payload_base64);

		$signature = hash_hmac("sha512", "{$header_base64}.{$payload_base64}", JWT_SECRET, true);
		return self::b64_custom_encode($signature);
	}

	public function saveToCookie(string $name): void {
		$this->updateTime();
		setcookie($name, "", time() - 3600);
		setcookie($name, $this->export(), time() + JWT_EXP);
		return;
	}

	private static function b64_custom_encode(string $msg): string {
		$out = base64_encode($msg);
		return str_replace("=", "", strtr($out, "+/", "-_"));

	}
}
?>
