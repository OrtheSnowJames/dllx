package dllx

import (
	"archive/zip"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"plugin"
	"runtime"
)

type Manifest struct {
	Name      string            `json:"name"`
	Platforms map[string]string `json:"platforms"`
}

// Extracts the .dllx (zip file) into a temporary directory
func extractDllx(dllxFile string, targetDir string) error {
	// Open the dllx file (which is a zip archive)
	zipReader, err := zip.OpenReader(dllxFile)
	if err != nil {
		return fmt.Errorf("failed to open zip file: %v", err)
	}
	defer zipReader.Close()

	// Extract all files in the zip archive to the target directory
	for _, file := range zipReader.File {
		destPath := filepath.Join(targetDir, file.Name)
		if file.FileInfo().IsDir() {
			os.MkdirAll(destPath, os.ModePerm)
		} else {
			outFile, err := os.Create(destPath)
			if err != nil {
				return fmt.Errorf("failed to create file %s: %v", destPath, err)
			}
			defer outFile.Close()

			zipFile, err := file.Open()
			if err != nil {
				return fmt.Errorf("failed to open file inside zip: %v", err)
			}
			defer zipFile.Close()

			_, err = outFile.ReadFrom(zipFile)
			if err != nil {
				return fmt.Errorf("failed to extract file: %v", err)
			}
		}
	}
	return nil
}

// Read the manifest.json inside the .dllx and determine the platform-specific file to extract
func readManifestFromDllx(dllxFile string) (Manifest, error) {
	// Open the .dllx zip archive
	zipReader, err := zip.OpenReader(dllxFile)
	if err != nil {
		return Manifest{}, fmt.Errorf("failed to open zip file: %v", err)
	}
	defer zipReader.Close()

	// Search for the manifest.json file in the archive
	var manifestFile *zip.File
	for _, file := range zipReader.File {
		if file.Name == "manifest.json" {
			manifestFile = file
			break
		}
	}

	if manifestFile == nil {
		return Manifest{}, fmt.Errorf("manifest.json not found in .dllx file")
	}

	// Open the manifest.json file
	manifestReader, err := manifestFile.Open()
	if err != nil {
		return Manifest{}, fmt.Errorf("failed to open manifest.json: %v", err)
	}
	defer manifestReader.Close()

	// Decode the JSON data into the Manifest struct
	var manifest Manifest
	decoder := json.NewDecoder(manifestReader)
	if err := decoder.Decode(&manifest); err != nil {
		return Manifest{}, fmt.Errorf("failed to decode manifest.json: %v", err)
	}

	return manifest, nil
}

// Load the appropriate library based on the platform
func loadLibrary(dllxFile string, manifest Manifest) (*plugin.Plugin, error) {
	// Detect platform
	currentPlatform := runtime.GOOS
	var platformFilePath string

	switch currentPlatform {
	case "windows":
		platformFilePath = manifest.Platforms["windows"]
	case "darwin": // macOS
		platformFilePath = manifest.Platforms["macos"]
	case "linux":
		platformFilePath = manifest.Platforms["linux"]
	case "ios":
		platformFilePath = manifest.Platforms["ios"]
	case "android":
		platformFilePath = manifest.Platforms["android"]
	default:
		return nil, fmt.Errorf("unsupported platform: %s", currentPlatform)
	}

	// Extract the .dllx file contents into a directory
	err := extractDllx(dllxFile, "./extracted") // Adjust your file path here
	if err != nil {
		return nil, err
	}

	// Load the plugin dynamically (this assumes a .so, .dylib, or .dll file format)
	pluginFile := filepath.Join("./extracted", platformFilePath)
	return plugin.Open(pluginFile)
}