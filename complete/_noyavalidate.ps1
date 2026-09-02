
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'noyavalidate' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'noyavalidate'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'noyavalidate' {
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Validate each document against the JSON Schema 2020-12 at PATH (the schema may itself be YAML or JSON)')
            [CompletionResult]::new('--schema', '--schema', [CompletionResultType]::ParameterName, 'Validate each document against the JSON Schema 2020-12 at PATH (the schema may itself be YAML or JSON)')
            [CompletionResult]::new('--fix', '--fix', [CompletionResultType]::ParameterName, 'Rewrite FILE in place via the CST formatter (lossless: byte-faithful for everything except normalised whitespace and line endings). With stdin input, the formatted bytes go to stdout')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Suppress success output')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Suppress success output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
