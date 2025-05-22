# PowerShell script to convert leftimage.jpg to app_icon.ico
# This requires PowerShell and the System.Drawing assembly

Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms

# Check if the source image exists
$sourceImage = "src/leftimage.jpg"
if (-not (Test-Path $sourceImage)) {
    Write-Error "Source image not found: $sourceImage"
    exit 1
}

# Create a temporary bitmap file
$tempBmpPath = [System.IO.Path]::GetTempFileName() + ".bmp"
$iconPath = "app_icon.ico"

try {
    # Load the image and save as a bitmap
    $image = [System.Drawing.Image]::FromFile((Resolve-Path $sourceImage))

    # Create bitmaps of different sizes for the icon (Windows icons typically have multiple sizes)
    $sizes = @(16, 32, 48, 64, 128, 256)
    $bitmaps = @()

    foreach ($size in $sizes) {
        $bmpSize = [System.Drawing.Size]::new($size, $size)
        $bitmap = New-Object System.Drawing.Bitmap $image, $bmpSize
        $bitmaps += $bitmap
    }

    # Save the largest bitmap to a temporary file
    $bitmaps[-1].Save($tempBmpPath, [System.Drawing.Imaging.ImageFormat]::Bmp)

    # Use the Windows Forms icon creation functionality
    $icon = [System.Drawing.Icon]::FromHandle(([System.Drawing.Bitmap]::new($tempBmpPath)).GetHicon())

    # Save the icon
    $fileStream = [System.IO.File]::Create($iconPath)
    $icon.Save($fileStream)
    $fileStream.Close()

    Write-Host "Icon created successfully: $iconPath"
    Write-Host "Now you can build your application with 'cargo build --release' to include the icon in the executable."
}
catch {
    Write-Error "Error creating icon: $_"
}
finally {
    # Clean up resources
    if ($icon) { $icon.Dispose() }
    if ($image) { $image.Dispose() }
    foreach ($bitmap in $bitmaps) {
        if ($bitmap) { $bitmap.Dispose() }
    }

    # Remove temporary file
    if (Test-Path $tempBmpPath) {
        Remove-Item $tempBmpPath -Force
    }
}
