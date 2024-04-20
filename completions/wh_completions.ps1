Register-ArgumentCompleter -Native -CommandName wh -ScriptBlock {
	param($word, $ast, $position)

	$word = "$word"
	$command = $ast.Extent.Text
	$prev = ""
	if($command.length -lt $position) {
		$prev = $command.TrimEnd().Split(" ") | Select-Object -Last 1
	} else {
		$s = $command.Substring(0, $position)
		$i = $s.LastIndexOf(" ")
		if($i -ge 0) {
			$prev = $s.Substring(0, $i).TrimEnd().Split(" ") | Select-Object -Last 1
		}
	}
	$global:prev = $prev

	if($prev -ceq "--completions") {
		$shells = @("powershell")

		foreach($s in $shells) {
			if($s -like "$word*") {
				$s
			}
		}

		return
	}

	if($word.StartsWith("--")) {
		$longs = @("--all", "--completions", "--help", "--no-pathext", "--show-dot", "--show-tilde", "--skip-dot", "--skip-tilde", "--version")
		foreach($s in  $longs) {
			if($s -clike "$word*") {
				$s
			}
		}

		return
	}

	if($word.StartsWith("-")) {
		$chars = $word.Substring(1)
		foreach($char in "ahPV".GetEnumerator()) {
			if(!$chars.Contains($char)) {
				"-$chars$char"
			}
		}

		return
	}

	# It's not a flag
	if(!$IsWindows) {
		Get-Command -CommandType Application -Name "$word*" | Select-Object -Unique -ExpandProperty Name | Sort-Object
		return
	}

	$dotIndex = $word.LastIndexOf(".")
	$ext = ""
	if($dotIndex -gt 0) {
		$ext = $word.Substring($dotIndex)
	}

	Get-Command -ea ignore -CommandType Application -name "$word*" `
	| Foreach-Object {
		$i = $_.name.LastIndexOf(".")
		if((!$ext -or $_.Name.Substring($i) -notlike "$ext*") -and $i -gt 0) {
			$_.Name.Substring(0, $i)
		} else {
			$_.Name
		}
	}
	| Sort-Object -Unique
}
