
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'noyafmt' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'noyafmt'
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
        'noyafmt' {
            [CompletionResult]::new('--indent', '--indent', [CompletionResultType]::ParameterName, 'Indentation width in spaces')
            [CompletionResult]::new('--check', '--check', [CompletionResultType]::ParameterName, 'Verify each FILE is formatted; print the list of files that need formatting and exit 1 if any do. Non-destructive. Suitable as a pre-commit / CI gate')
            [CompletionResult]::new('--write', '--write', [CompletionResultType]::ParameterName, 'Rewrite each FILE in place. Default is to print the formatted source to stdout')
            [CompletionResult]::new('--stdin', '--stdin', [CompletionResultType]::ParameterName, 'Read from stdin, write to stdout. Mutually exclusive with FILE arguments')
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
