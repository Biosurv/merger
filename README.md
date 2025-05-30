# Merger App

## Installation
To install the app, go to the **latest release** on the repository page.

## Usage

### Overview
The Merger App allows users to merge lab and epidemiological (Epi) information into a detailed run report. 

<p align="center">
<img src="Python Version/Merger_before_interface.png" alt="Interface" height=70% width=70%>
</p>

### Steps to Use
1. **Insert Run Values**
   - The run values such as Run Number, FlowCell ID can be filled in during merge with EpiInfo. No errors will occur if the values are not filled in. There are drop down menus for the Primers, these will always add the defaults, make sure to choose the right one.

2. **Select Input Files**
   - Each input file can be selected individually or they can be placed in the dropbox.
   - Only CSV files with the correct column headers are accepted. Otherwise, an error message will be displayed.

3. **Select Destination Directory**
   - Choose a directory where the `barcodes.csv` file will be saved.

4. **Merge CSVs**
   - Click the "Merge CSVs" button to initiate the merging process.
   - If errors occur, an error message will provide details on the issue.
   - A common error is missing sample IDs in either file. Ensure all IDs in the Lab Info sheet are present in the Epi Info sheet. The Epi Info sheet may contain extra IDs, but the Lab Info sheet must not have missing IDs.
    - The output at the chosen destination is `barcodes.csv` is used as input for Piranha.

5. **Error Handling**
   - The app should handle most errors and display an error message, such as missing expected columns or missing input / destination
   - If the app crashes without displaying an error message, a log file called `merger_app_error.log` will be created in the chosen destionation. Please upload the log file to this repositorys' issue page.

6. **Language Options**
   - Users can switch between English, French

7. **Generate Template Files**
   - Users can generate template files for `samples.csv` which contains all the nescessary Headers.

## Input
You will need a CSV version of the EpiInfo database, this can be downloaded by the Lab Data manager, there is no need to format the file; just make sure all the relevant headers are present. The app will let you know if headers are missing. 
Next is the sample_template, which can be generated within the app, please fill the sample and barcode columns before using merge. The app will then add the EpiInfo and Run values. 

## Output
The app generates the `[Run Number]_barcodes.csv` (Run Number inputted into the interface textbox) at your chosen destination containing:
- Merged Lab and Epi Info headers.
- Any added values from the interface will be filled in the output
- Certain headers, such as RunQC, QCComments, and certain date columns have to be filled in by hand and will be left blank in the output.

## Compiling the app

If you want to compile the app yourself, perhaps for a OS that isn't currently supported. This app was compiled using Nuitka and in the misc folder, there is the conda environment YAML that was used to build the app. You can clone this repository if you want to compile the app:

1. Create the Conda/Mamba environment using:
```
[conda|mamba] env create -f extras/nuitka.yml
```
2. Activate the environment:
```
[conda|mamba] activate nuitka
```
4. Use the compilation command:
```
nuitka --onefile --enable-plugins=pyqt5 --include-data-files=Logo.png=./Logo.png --include-data-files=Icon.ico=./Icon.ico --disable-console --windows-icon-from-ico=Icon.ico --company-name="Biosurv International" --product-name="CSV Merger Application" --file-version=2.1.0 --file-description=="This App merges LabID and EpiID files to a standard output"  Merger_before_piranha.py
```
Make sure that you are the Python Version folder.