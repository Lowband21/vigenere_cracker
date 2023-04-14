# Vigenere Cracker
Vigenere Cracker is a command-line tool that decrypts text encrypted with the Vigenère cipher. This tool implements various techniques to analyze and crack the cipher, including the chi-squared test and mutual index of coincidence. It provides a confidence level for the decryption, helping you determine the most likely key and plaintext.

## Getting Started
These instructions will help you set up and run the Vigenere Cracker on your local machine.

### Prerequisites
- Rust programming language (install from https://rustup.rs/)

### Running the Project
1. Clone the repository to your local machine:

```bash
git clone https://github.com/Lowband21/vigenere_cracker.git
```
2. Navigate to the project directory:
```
cd vigenere_cracker
```
3. Build the project:
```
cargo build --release
```
4. Run the tool:

```
./target/release/vigenere_cracker
```
5. Select a file from prompt and press enter

## Performance:
On my laptop I am able to decrypt all the files found in the input folder in just 300ms.
With most of that time being taken by the decryption of Michael's long independence text at 141ms.
Smaller texts typically take under 50ms.


## Additional Notes
- The confidence level provided by the tool is based on the mutual index of coincidence (MIC) between the decrypted text and the English language frequencies.
- It is recommended to use a sample of the ciphertext with a minimum length of 100 characters for accurate key length detection and decryption.
- The LOG_LEVEL can be modified in the file logger.rs

## License
This project is licensed under the MIT License.
