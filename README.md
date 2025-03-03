# Merger App

## Installation
To install the app, go to the **latest release** on the repository page.

## Usage

### Overview
The Merger App allows users to merge lab and epidemiological (Epi) information into a detailed run report. 

![image](https://github.com/user-attachments/assets/b3c282b8-7530-49c0-a1e9-ac568493e5b9)

### Steps to Use
1. **Insert Run Name** (1)
   - The run name is required and will be used to generate the output file in the format: `Run Number_detailed_run_report.csv`.

2. **Select Input Files** (2)
   - Each input file must be selected individually.
   - Only CSV files with the correct column headers are accepted. Otherwise, an error message will be displayed.

3. **Select Destination Directory**
   - Choose a directory where the `barcodes.csv` file will be saved.

4. **Merge CSVs** (3)
   - Click the "Merge CSVs" button to initiate the merging process.
   - If errors occur, an error message will provide details on the issue.
   - A common error is missing sample IDs in either file. Ensure all IDs in the Lab Info sheet are present in the Epi Info sheet. The Epi Info sheet may contain extra IDs, but the Lab Info sheet must not have missing IDs.

5. **Error Handling**
   - If the app crashes without displaying an error message, report the issue along with the input files to Biosurv International for troubleshooting.

6. **Language Options** (4)
   - Users can switch between English, French, and Portuguese.

7. **Generate Template Files** (5)
   - Users can generate template files for `barcodes.csv` and `Lab Info`.
   - `barcodes.csv` is used as input for Piranha.
   - `Lab Info` is needed for the Merger App.

## Output
The app generates a detailed run report containing:
- Merged Lab and Epi Info data.
- Additional empty columns for:
  - Quality Control (QC) review.
  - VDPV emergence group information.


