param(
    [Parameter(Mandatory=$true, Position=0)]
    [string]$Prompt,
    
    [Parameter(Mandatory=$false)]
    [string]$Model = "llama3.2:3b",
    
    [Parameter(Mandatory=$false)]
    [switch]$Raw = $true
)

function Stream-OllamaResponse {
    param(
        [string]$Prompt,
        [string]$Model,
        [bool]$Raw
    )

    $body = @{
        model = $Model
        prompt = $Prompt
        raw = $Raw
    } | ConvertTo-Json

    $request = [System.Net.WebRequest]::Create("http://localhost:11434/api/generate")
    $request.Method = "POST"
    $request.ContentType = "application/json"

    $bytes = [System.Text.Encoding]::UTF8.GetBytes($body)
    $request.ContentLength = $bytes.Length
    $requestStream = $request.GetRequestStream()
    $requestStream.Write($bytes, 0, $bytes.Length)
    $requestStream.Close()

    try {
        $response = $request.GetResponse()
        $responseStream = $response.GetResponseStream()
        $reader = New-Object System.IO.StreamReader($responseStream)

        $colors = @("Green", "Cyan", "Yellow", "Magenta", "Blue", "DarkCyan", "DarkGreen", "DarkMagenta", "DarkYellow", "White")
        $colorIndex = 0

        while (-not $reader.EndOfStream) {
            $line = $reader.ReadLine()
            
            if ([string]::IsNullOrWhiteSpace($line)) { continue }
            
            try {
                $jsonObj = ConvertFrom-Json $line
                
                if ($jsonObj.response) {
                    Write-Host $jsonObj.response -NoNewline -ForegroundColor $colors[$colorIndex]
                    $colorIndex = ($colorIndex + 1) % $colors.Length
                }
                
                if ($jsonObj.done) {
                    Write-Host ""
                }
            }
            catch {
                Write-Host $line -ForegroundColor Red
            }
        }

        $reader.Close()
        $response.Close()
    }
    catch {
        Write-Host "Error communicating with Ollama API: $_" -ForegroundColor Red
    }
}

Write-Host "Sending prompt to $Model`: " -ForegroundColor Magenta
Write-Host "`"$Prompt`"" -ForegroundColor White
Write-Host ""

Stream-OllamaResponse -Prompt $Prompt -Model $Model -Raw $Raw
