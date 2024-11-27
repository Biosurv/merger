import traceback
import sys
import os
import pandas as pd
from itertools import product
from datetime import datetime

from PyQt5.QtCore import Qt
from PyQt5.QtGui import QPixmap, QFont, QIcon
from PyQt5.QtWidgets import QApplication, QMainWindow, QLineEdit, QPushButton, QMessageBox, QLabel, QFileDialog, QComboBox, QWidget, QVBoxLayout, QScrollArea

# AUTHOR - Shean Mobed
# ORG - Biosurv International

# DESCRIPTION
"""
The Merger app takes three files, output csv file for Piranha analysis pipeline, a lab info csv file containing required laboratory info, and an epi info csv file that was
pulled from the EpiInfo database of the PolioLab. The User inputs a run number and clicks the Merge button and the output will be published in the Downloads folder. The app has
three built in languages: English, French, and Portuguese. The app also has a feature to generate template files for labinfo that should be filled in by the user.
"""

# COMPILE COMMANDS
"""
CLI MAC OS
python3.11 -m nuitka --macos-app-icon=Icon.icns --include-data-files=Logo.png=Logo.png
--include-data-files=Icon.ico=Icon.icns --macos-app-mode=gui --disable-console --macos-create-app-bundle
--macos-app-name="CSV Merger App" --onefile --enable-plugin=pyqt5 Merger_2.0.2.py

WIN - CommandPrompt
nuitka --onefile --enable-plugins=pyqt5 --include-data-files=Logo.png=./Logo.png --include-data-files=Icon.ico=./Icon.ico --disable-console 
--windows-icon-from-ico=Icon.ico --company-name="Biosurv International" --product-name="CSV Merger Application" --file-version=2.0.2 
--file-description=="This App merges LabID and EpiID files to a standard output"  Merger_2.0.2.py
"""

# CHANGELOG 
"""
v 2.0.1 --> v 2.0.2
- fixed missing indent causing the the app to think there were no files even if they were selected, line 400
- added statement that checks if key error encountered after checking the three files, and if so stops the concat function, line 448 - 493
"""

# APP
class App(QMainWindow):
    def __init__(self):
        super().__init__()
        
        global screen_width
        global screen_height

        screen_size = self.screen().size()
        screen_width = int(screen_size.width() * 0.5)
        screen_height = int(screen_size.height() * 0.5)
        self.resize(screen_width, screen_height)  # 1200,1000 original

        self.setStyleSheet("background-color: white; color: black;")  # Set window color to white

        self.setWindowIcon(
            QIcon(os.path.join(os.path.dirname(__file__), "Icon.ico")))  # Sets top left icon to custom icon
        self.setWindowTitle("CSV Merger App")  # App Title
        
        # ---- VERSION ---- # change when new veriosn
        self.app_ver = QLabel('Version: 2.0.2',self)
        self.app_ver.setStyleSheet("background-color:transparent")
        self.app_ver.setGeometry(int(screen_width * 0.88), int(screen_height * 0.95), int(screen_width * 0.2), int(screen_height * 0.06))
        self.app_ver.setFont(QFont('Arial', 9))

        # ---- LOGO ----
        self.logo_label = QLabel(self)
        self.logo_label.setGeometry(int(screen_width * 0.15), int(screen_height * 0.08), int(screen_width * 0.5), int(screen_height * 0.2)) # (x_position, y_postion, x_dim, y_dim)
        pixmap = QPixmap(os.path.join(os.path.dirname(__file__), 'Logo.png'))
        self.logo_label.setPixmap(pixmap)
        self.logo_label.setScaledContents(True)
        
        # ---- RUN NUMBER BOX ----
        self.runnumber_textbox = QLineEdit(self)
        self.runnumber_textbox.setGeometry(int(screen_width * 0.18), int(screen_height * 0.31), int(screen_width * 0.3), int(screen_height * 0.05))
        self.runnumber_textbox.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style:solid; border-width: 2px;border-radius: 10px;") 
        
        self.runnumber_label= QLabel("Run Number:",self)
        self.runnumber_label.setStyleSheet("background-color:transparent")
        self.runnumber_label.setGeometry(int(screen_width * 0.055), int(screen_height * 0.3), int(screen_width * 0.2), int(screen_height * 0.06))
        self.runnumber_label.setFont(QFont('Arial', 9))
        
        
        
        # ---- BUTTONS ----
        self.btn_concat = QPushButton('Merge CSVs', self)
        self.btn_concat.setGeometry(int(screen_width * 0.31), int(screen_height * 0.67), int(screen_width * 0.15),
                                    int(screen_height * 0.1))  # button position and dimension 60, 770, 300, 100
        self.btn_concat.setFont(QFont('Arial', 9))
        self.btn_concat.setStyleSheet("QPushButton"
                                      "{"
                                      "color: white; border-radius: 15px;background-color:#2e3192;"
                                      "border-color:black;border-style: solid;border-width: 1px;"
                                      "}"
                                      "QPushButton::pressed"
                                      "{"
                                      "background-color : #3638d8;"
                                      "}")
        self.btn_concat.clicked.connect(self.concatenate_csv)

        self.btn_clear = QPushButton('Clear', self)
        self.btn_clear.setGeometry(int(screen_width * 0.51), int(screen_height * 0.67), int(screen_width * 0.15),
                                   int(screen_height * 0.1))  # button position and dimension 400, 770, 300, 100
        self.btn_clear.setFont(QFont('Arial', 9))
        self.btn_clear.setStyleSheet("QPushButton"
                                     "{"
                                     "color: white; border-radius: 15px;background-color:#2e3192;"
                                     "border-color:black;border-style: solid;border-width: 1px;"
                                     "}"
                                     "QPushButton::pressed"
                                     "{"
                                     "background-color : #3638d8;"
                                     "}")
        self.btn_clear.clicked.connect(self.clear_list)
        
        # ---- TEMPLATE ----
        ## BOX
        self.template_combobox = QComboBox(self)
        self.template_combobox.addItems(['Barcodes', 'LabInfo'])
        self.template_combobox.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style: "
                                             "solid;border-width: 2px;border-radius: 10px;")
        self.template_combobox.setGeometry(int(screen_width * 0.25), int(screen_height * 0.85), int(screen_width * 0.1),
                                           int(screen_height * 0.06))
        ## LABEL
        self.template_label = QLabel('Generate Template:', self)
        self.template_label.setStyleSheet("background-color:transparent")
        self.template_label.setGeometry(int(screen_width * 0.08), int(screen_height * 0.85), int(screen_width * 0.2), int(screen_height * 0.06))
        self.template_label.setFont(QFont('Arial', 10))
        ## BUTTON
        self.btn_template = QPushButton('Create', self)
        self.btn_template.setGeometry(int(screen_width * 0.37), int(screen_height * 0.85), int(screen_width * 0.08),int(screen_height * 0.06))
        self.btn_template.setFont(QFont('Arial', 8))
        self.btn_template.setStyleSheet("QPushButton"
                                     "{"
                                     "color: white; border-radius: 15px;background-color:#2e3192;"
                                     "border-color:black;border-style: solid;border-width: 1px;"
                                     "}"
                                     "QPushButton::pressed"
                                     "{"
                                     "background-color : #3638d8;"
                                     "}")
        self.btn_template.clicked.connect(self.generate_template)
        
        
        # ---- LANGUAGE COMBO BOX ----
        self.lang_combobox = QComboBox(self)
        self.lang_combobox.addItems(['English', 'Francais','Português'])
        self.lang_combobox.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style: "
                                             "solid;border-width: 2px;border-radius: 10px;")
        self.lang_combobox.setGeometry(int(screen_width * 0.7), int(screen_height * 0.15), int(screen_width * 0.1),int(screen_height * 0.06))
        
        self.lang_combobox.currentIndexChanged.connect(self.update_language)
        
        self.lang_label = QLabel('Language', self)
        self.lang_label.setStyleSheet("background-color:transparent")
        self.lang_label.setGeometry(int(screen_width * 0.7), int(screen_height * 0.1), int(screen_width * 0.2), int(screen_height * 0.06))
        self.lang_label.setFont(QFont('Arial', 10))

        # ---- DESTINATION ----
        ## BOX
        self.destination_entry = QLineEdit(self)
        self.destination_entry.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style: "
                                             "dashed;border-width: 2px;border-radius: 10px;")
        self.destination_entry.setGeometry(int(screen_width * 0.18), int(screen_height * 0.58), int(screen_width * 0.63),
                                           int(screen_height * 0.06))  # 300, 670, 750, 70
        self.destination_entry.setFont(QFont('Arial', 11))
        ## LABEL
        self.destination_label = QLabel('Destination:', self)
        self.destination_label.setStyleSheet("background-color:transparent")
        self.destination_label.setGeometry(int(screen_width * 0.055), int(screen_height * 0.58), int(screen_width * 0.2), int(screen_height * 0.05))
        self.destination_label.setFont(QFont('Arial', 9))
        ## BUTTON
        self.destination_btn = QPushButton('Select', self)
        self.destination_btn.setGeometry(int(screen_width * 0.82), int(screen_height * 0.58), int(screen_width * 0.1), int(screen_height * 0.06))
        self.destination_btn.setFont(QFont('Arial', 8))
        self.destination_btn.setStyleSheet("QPushButton"
                                           "{"
                                           "color: white; border-radius: 15px;background-color:#2e3192;"
                                           "border-color:black;border-style: solid;border-width: 1px;"
                                           "}"
                                           "QPushButton::pressed"
                                           "{"
                                           "background-color : #3638d8;"
                                           "}")
        
        self.destination_btn.clicked.connect(lambda: self.select_destination(3))

        # ---- EPIINFO ----
        ## BOX
        self.epi_entry = QLineEdit(self)
        self.epi_entry.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style: "
                                             "dashed;border-width: 2px;border-radius: 10px;")
        self.epi_entry.setGeometry(int(screen_width * 0.18), int(screen_height * 0.51), int(screen_width * 0.63), int(screen_height * 0.06))
        self.epi_entry.setFont(QFont('Arial', 11))
        ## LABEL
        self.epi_label = QLabel('Epi Info:', self)
        self.epi_label.setStyleSheet("background-color:transparent")
        self.epi_label.setGeometry(int(screen_width * 0.055), int(screen_height * 0.51), int(screen_width * 0.2), int(screen_height * 0.05)) 
        self.epi_label.setFont(QFont('Arial', 9))
        ## BUTTON
        self.epi_selec_btn = QPushButton('Select', self)
        self.epi_selec_btn.setGeometry(int(screen_width * 0.82), int(screen_height * 0.51), int(screen_width * 0.1), int(screen_height * 0.06))
        self.epi_selec_btn.setFont(QFont('Arial', 8))
        self.epi_selec_btn.setStyleSheet("QPushButton"
                                           "{"
                                           "color: white; border-radius: 15px;background-color:#2e3192;"
                                           "border-color:black;border-style: solid;border-width: 1px;"
                                           "}"
                                           "QPushButton::pressed"
                                           "{"
                                           "background-color : #3638d8;"
                                           "}")
        self.epi_selec_btn.clicked.connect(lambda: self.select_destination(1))
        
        # ---- LABINFO ----
        ## BOX
        self.lab_entry = QLineEdit(self)
        self.lab_entry.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style: "
                                             "dashed;border-width: 2px;border-radius: 10px;")
        self.lab_entry.setGeometry(int(screen_width * 0.18), int(screen_height * 0.44), int(screen_width * 0.63),
                                           int(screen_height * 0.06))
        self.lab_entry.setFont(QFont('Arial', 11))
        ## LABEL
        self.lab_label = QLabel('Lab Info:', self)
        self.lab_label.setStyleSheet("background-color:transparent")
        self.lab_label.setGeometry(int(screen_width * 0.055), int(screen_height * 0.44), int(screen_width * 0.2),
                                           int(screen_height * 0.05))  # 100, 670, 200, 70
        self.lab_label.setFont(QFont('Arial', 9))
        ## BUTTON
        self.lab_selec_btn = QPushButton('Select', self)
        self.lab_selec_btn.setGeometry(int(screen_width * 0.82), int(screen_height * 0.44), int(screen_width * 0.1), int(screen_height * 0.06))  # 750, 770, 300, 100
        self.lab_selec_btn.setFont(QFont('Arial', 8))
        self.lab_selec_btn.setStyleSheet("QPushButton"
                                           "{"
                                           "color: white; border-radius: 15px;background-color:#2e3192;"
                                           "border-color:black;border-style: solid;border-width: 1px;"
                                           "}"
                                           "QPushButton::pressed"
                                           "{"
                                           "background-color : #3638d8;"
                                           "}")
        self.lab_selec_btn.clicked.connect(lambda: self.select_destination(2))
        
         # ---- PIRANHA ----
         ## BOX
        self.pir_entry = QLineEdit(self)
        self.pir_entry.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style: "
                                             "dashed;border-width: 2px;border-radius: 10px;")
        self.pir_entry.setGeometry(int(screen_width * 0.18), int(screen_height * 0.37), int(screen_width * 0.63), int(screen_height * 0.06))
        self.pir_entry.setFont(QFont('Arial', 11))
        ## LABEL
        self.pir_label = QLabel('Piranha Report:', self)
        self.pir_label.setStyleSheet("background-color:transparent")
        self.pir_label.setGeometry(int(screen_width * 0.055), int(screen_height * 0.37), int(screen_width * 0.2), int(screen_height * 0.05))
        self.pir_label.setFont(QFont('Arial', 9))
        ## BUTTON
        self.pir_selec_btn = QPushButton('Select', self)
        self.pir_selec_btn.setGeometry(int(screen_width * 0.82), int(screen_height * 0.37), int(screen_width * 0.1), int(screen_height * 0.06))
        self.pir_selec_btn.setFont(QFont('Arial', 8))
        self.pir_selec_btn.setStyleSheet("QPushButton"
                                           "{"
                                           "color: white; border-radius: 15px;background-color:#2e3192;"
                                           "border-color:black;border-style: solid;border-width: 1px;"
                                           "}"
                                           "QPushButton::pressed"
                                           "{"
                                           "background-color : #3638d8;"
                                           "}")
        self.pir_selec_btn.clicked.connect(lambda: self.select_destination(4))
        
    def update_language(self):
        # Dictionary
        translations = {
            "English": {
                "btn_clear": "Clear",
                "btn_concat": "Merge CSVs",
                "destination_label": "Destination:",
                "destination_btn": "Select",
                "epi_selec_btn": "Select",
                "lab_selec_btn": "Select",
                "pir_selec_btn": "Select",
                "template_label": "Generate Template:",
                "btn_template": "Create",
                "lang_label": "Language",
                "runnumber_label":"Run Number:"
            },
            "Francais": {
                "btn_clear": "Effacer",
                "btn_concat": "Rejoindre CSVs",
                "destination_label": "Destination:",
                "destination_btn": "Sélectionner",
                "epi_selec_btn": "Sélectionner",
                "lab_selec_btn": "Sélectionner",
                "pir_selec_btn": "Sélectionner",
                "template_label": "Générer un modèle:",
                "btn_template": "Créer",
                "lang_label": "Langue",
                "runnumber_label":"Numéro d'exécution:"
            },
            "Português": {
                "btn_clear": "Apagar",
                "btn_concat": "Unir CSVs",
                "destination_label": "Destino:",
                "destination_btn": "Selecionar",
                "epi_selec_btn": "Selecionar",
                "lab_selec_btn": "Selecionar",
                "pir_selec_btn": "Selecionar",
                "template_label": "Gerar modelo:",
                "btn_template": "Criar",
                "lang_label": "Idioma",
                "runnumber_label":"Número de execução:"
            }
        }

        # Updates widget text based off option
        selected_language = self.lang_combobox.currentText()
        if selected_language in translations:
            for widget_attr, text in translations[selected_language].items():
                widget = getattr(self, widget_attr)
                widget.setText(text)
                
        if selected_language == 'English':
            self.runnumber_textbox.setGeometry(int(screen_width * 0.18), int(screen_height * 0.31), int(screen_width * 0.3), int(screen_height * 0.05))

        else:
            self.runnumber_textbox.setGeometry(int(screen_width * 0.23), int(screen_height * 0.31), int(screen_width * 0.3), int(screen_height * 0.05))


    def select_destination(self, type):
        file_dialog = QFileDialog()
        file_dialog.setFileMode(QFileDialog.AnyFile if type in (1, 2, 4) else QFileDialog.Directory)
        
        if file_dialog.exec_():
            selected_files = file_dialog.selectedFiles()
            if selected_files:
                selected_file = selected_files[0]
                selected_file_ext = selected_file.split('.')[-1].upper()
                
                # Only csv for input apart from destination selection
                if type in (1, 2, 4) and not selected_file.endswith('.csv'):
                    if self.lang_combobox.currentText() == 'English':
                        QMessageBox.warning(self, "Invalid File", f"You selected a .{selected_file_ext} file type. Please select a .CSV file.")
                        return

                    if self.lang_combobox.currentText() == 'Francais':
                        QMessageBox.warning(self, "Fichier invalide", f"Vous avez sélectionné un fichier de type .{selected_file_ext}. Veuillez sélectionner un fichier .CSV.")
                        return

                    if self.lang_combobox.currentText() == 'Português':
                        QMessageBox.warning(self, "Ficheiro  inválido", f"Você selecionou um arquivo do tipo .{selected_file_ext}. Por favor, selecione um arquivo .CSV.")
                        return
                
                # labinfo
                if type == 1:
                    self.epi_entry.setText(selected_file)
                # epiinfo
                elif type == 2:
                    self.lab_entry.setText(selected_file)
                # destination
                elif type == 3:
                    self.destination_entry.setText(selected_file)
                # piranha report
                elif type == 4:
                    self.pir_entry.setText(selected_file)

    def generate_template(self):
        
        selected_option = self.template_combobox.currentText()
        print(selected_option)
        
        # Saving template to downloads
        downloads_folder = os.path.join(os.path.expanduser("~"), "Downloads")
        
        if selected_option == "Barcodes":
            # Generates barcodes.csv template, with prefilled barcode and well combinations
            barcode_list = [f'barcode{str(n).zfill(2)}' for n in range(1, 97)]
            barcodes_template = pd.DataFrame({'sample': '', 'barcode': barcode_list})
            
            file_path = os.path.join(downloads_folder, "template_barcodes.csv")
            barcodes_template.to_csv(file_path, index=False)
            QMessageBox.information(self,'Saved','Barcodes Template saved to Downloads')
            
        else:
            well_list = [f'{chr(row)}{str(col).zfill(2)}' for row, col in product(range(65, 73), range(1, 13))]
            labinfo_cols = "labid,barcode,IsQCRetest,IfRetestOriginalRun,SequencingLab,SampleType,DelaysInProccessingForDDNS,DetailsOfDelays,DateRNAextraction,ExtractionKit,ExtractionType,DateRTPCR,RTPCRMachine,RTPCRcomments,DatePanEVPCR,PanEVPCRMachine,PanEVprimers,PanEVPCRcomments,DateVP1PCR,VP1PCRMachine,VP1primers,VP1PCRcomments,PositiveControlPCRCheck,NegativeControlPCRCheck,LibraryPreparationKit,Well,RunNumber,DateSeqRunLoaded,SequencerUsed,FlowCellVersion,FlowCellID,FlowCellPriorUses,PoresAvilableAtFlowCellCheck,MinKNOWSoftwareVersion,RunHoursDuration".split(',')
            labinfo_template = pd.DataFrame(columns=labinfo_cols, data={'well':well_list})
            file_path = os.path.join(downloads_folder, "template_labinfo.csv")
            labinfo_template.to_csv(file_path, index=False)
            QMessageBox.information(self,'Saved','LabInfo Template saved to Downloads')
            
            
                           
    def concatenate_csv(self):
        destination_path = self.destination_entry.text()
        epi_path = self.epi_entry.text()
        lab_path = self.lab_entry.text()
        pir_path = self.pir_entry.text()
                
        missing_files = []
            
            # Check each input path and collect missing files
        for input_path, file in [(destination_path, 'Destination'), (epi_path, 'Epi Info'), (lab_path, 'Lab Info'), (pir_path, 'Piranha Report')]:
            if input_path == '':
                if file != 'Destination':
                # Append the correct file name based on the language
                    if self.lang_combobox.currentText() == 'English':
                        file += ' File'
                    elif self.lang_combobox.currentText() == 'Francais':
                        file = 'Fichier ' + file
                    elif self.lang_combobox.currentText() == 'Português':
                        file = 'Ficheiro ' + file
                else:
                    if self.lang_combobox.currentText() == 'Português':
                        file = 'Destino'
                        
                missing_files.append(file)
    
        # If there are any missing files, display a warning
        if missing_files:
            missing_files_msg = ',\n'.join(missing_files)               
            if self.lang_combobox.currentText() == 'English':
                QMessageBox.warning(self, 'Warning', f"Please select:\n{missing_files_msg}")
            elif self.lang_combobox.currentText() == 'Francais':
                QMessageBox.warning(self, 'Attention', f"Veuillez sélectionner:\n{missing_files_msg}")
            elif self.lang_combobox.currentText() == 'Português':
                QMessageBox.warning(self, 'Aviso', f"Por favor, selecione:\n{missing_files_msg}")
            return
                        
        # Read in files                
        epiinfo = pd.read_csv(epi_path, engine='python',sep=None, encoding='utf-8', encoding_errors='replace')
        lablist = pd.read_csv(lab_path, engine='python',sep=None, encoding='utf-8', encoding_errors='replace')
        piranha = pd.read_csv(pir_path, engine='python',sep=None, encoding='utf-8', encoding_errors='replace')
        
        def missing_column_warning(error_message, file):
            print(error_message)
            # Handles multiple missing columns
            if len(str(error_message).split(',')) > 1:
                missing = '\n'
                for _ in str(error_message).rsplit('\']')[0].split('"[\'')[1].split(','):
                    missing += (_.replace("'","").strip() + '\n')
            else: # single missing column
                missing = str(error_message).rsplit('\']')[0].split('"[\'')[1]
                
            # Dynamic Error Message
            if self.lang_combobox.currentText() == 'English':
                QMessageBox.critical(self, 'Error', f"These columns are missing from your {file} file: {missing}")
        
            elif self.lang_combobox.currentText() == 'Francais':
                QMessageBox.critical(self, 'Error', f"Ces colonnes sont absentes du fichier {file}: {missing}")
            
            else:
                QMessageBox.critical(self, 'Error', f"Estas colunas não constam do seu ficheiro {file}: {missing}")

            # Reset
            self.destination_entry.clear()
            self.epi_entry.clear()
            self.lab_entry.clear()
            self.pir_entry.clear()
            self.runnumber_textbox.clear()
            return
        
        # Epi Info Format checker and normalises non-utf8 characters
        key_error_encountered = False
        try:
            epi_fmt = "ICLabID;EpidNumber;CaseOrContact;Country;Province;District;StoolCondition;SpecimenNumber;DateOfOnset;DateStoolCollected;DateStoolSentfromField;DateStoolReceivedNatLevel;DateStoolSentToLab;DateStoolReceivedinLab;FinalCellCultureResult;DateFinalCellCultureResults;FinalITDResult;DateFinalrRTPCRResults;DateIsolateSentforSeq;SequenceName;DateSeqResult".split(';')
            epiinfo = epiinfo[epi_fmt]

            # Fixes non-utf8 characters
            def encode_col(series):
                 return series.str.normalize('NFKD').str.encode('ascii', errors='ignore').str.decode('utf-8')
            
            epiinfo = epiinfo.astype(str)
            epiinfo = epiinfo.apply(lambda x:encode_col(x))
            epiinfo = epiinfo.rename(columns={'ICLabID':'labid'})
            epiinfo['labid'] = epiinfo['labid'].astype("object")

        except KeyError as e:
                missing_column_warning(e,'EpiInfo')
                key_error_encountered = True
                
        # LabInfo Format Checker                       
        try:
            labinfo_fmt = "labid,barcode,IsQCRetest,IfRetestOriginalRun,SequencingLab,SampleType,DelaysInProccessingForDDNS,DetailsOfDelays,DateRNAextraction,ExtractionKit,ExtractionType,DateRTPCR,RTPCRMachine,RTPCRcomments,DatePanEVPCR,PanEVPCRMachine,PanEVprimers,PanEVPCRcomments,DateVP1PCR,VP1PCRMachine,VP1primers,VP1PCRcomments,PositiveControlPCRCheck,NegativeControlPCRCheck,LibraryPreparationKit,Well,RunNumber,DateSeqRunLoaded,SequencerUsed,FlowCellVersion,FlowCellID,FlowCellPriorUses,PoresAvilableAtFlowCellCheck,MinKNOWSoftwareVersion,RunHoursDuration".split(',')
            lablist = lablist[labinfo_fmt]
            
            # merge dataframes based on sample from lablist and ICLabID from epiinfo
            lablist.columns = lablist.columns.str.encode('ascii','ignore').str.decode('ascii')
            lablist[['labid','barcode']] = lablist[['labid','barcode']].astype("object")
            
        except KeyError as e:
                missing_column_warning(e,'LabInfo')
                key_error_encountered = True
                
        # Pirnha Report Format checker         
        try:
            piranha_fmt = "sample;barcode;institute;EPID;NonPolioEV|closest_reference;NonPolioEV|num_reads;NonPolioEV|nt_diff_from_reference;NonPolioEV|pcent_match;NonPolioEV|classification;Sabin1-related|closest_reference;Sabin1-related|num_reads;Sabin1-related|nt_diff_from_reference;Sabin1-related|pcent_match;Sabin1-related|classification;Sabin2-related|closest_reference;Sabin2-related|num_reads;Sabin2-related|nt_diff_from_reference;Sabin2-related|pcent_match;Sabin2-related|classification;Sabin3-related|closest_reference;Sabin3-related|num_reads;Sabin3-related|nt_diff_from_reference;Sabin3-related|pcent_match;Sabin3-related|classification;WPV1|closest_reference;WPV1|num_reads;WPV1|nt_diff_from_reference;WPV1|pcent_match;WPV1|classification;WPV2|closest_reference;WPV2|num_reads;WPV2|nt_diff_from_reference;WPV2|pcent_match;WPV2|classification;WPV3|closest_reference;WPV3|num_reads;WPV3|nt_diff_from_reference;WPV3|pcent_match;WPV3|classification;PositiveControl|closest_reference;PositiveControl|num_reads;PositiveControl|nt_diff_from_reference;PositiveControl|pcent_match;PositiveControl|classification;comments".split(';')
            piranha = piranha[piranha_fmt]
            piranha = piranha.rename(columns={'sample':'labid'})
            piranha[['labid','barcode']] = piranha[['labid','barcode']].astype("object")
            
        except KeyError as e:
                missing_column_warning(e,'Piranha Report')
                key_error_encountered = True
        
        # checks if one or more of the key error were encountered and escapes script
        if key_error_encountered:
            print("key_error_encountered")
            return
                
        # Check if sample and barcode columns in piranha and labinfo are the same   
        if piranha[['labid','barcode']].equals(lablist[['labid','barcode']]):
            print('piranha and labinfo are the same')
        else:
            if self.lang_combobox.currentText() == 'English':
                QMessageBox.critical(self, 'Error', f'The sample and/or barcode columns are not identical between the Piranha report and the LabInfo file.')
        
            elif self.lang_combobox.currentText() == 'Francais':
                QMessageBox.critical(self, 'Error', f'L\'erreur provient de la fusion des fichiers, Vérifiez que les deux fichiers ont des identifiants d\'échantillons!')
            
            else:
                QMessageBox.critical(self, 'Error', f'As colunas da amostra e/ou do código de barras não são idênticas entre o relatório Piranha e o ficheiro LabInfo.')

         
        # Merging all three files       
        try:
            # Merging and renaming
            merged_df = piranha.merge(lablist, how='left', on=['labid','barcode']).merge(epiinfo, how='left', on='labid')
            merged_df = merged_df.rename(columns={'labid':'sample'}).replace('nan','')
            
            # Add Extra Info columns
            extra_info = ['DateFastaGenerated','AnalysisPipelineVersion','RunQC','DDNSclassification','SampleQC','SampleQCChecksComplete','QCComments','ToReport','DateReported','EmergenceGroupVDPV1','EmergenceGroupVDPV2','EmergenceGroupVDPV3']
            merged_df[extra_info] = 12 * ''
           
            # Setting to final format before output
            final_fmt = "sample;barcode;EPID;institute;IsQCRetest;IfRetestOriginalRun;EpidNumber;SequencingLab;SampleType;CaseOrContact;Country;Province;District;StoolCondition;SpecimenNumber;DateOfOnset;DateStoolCollected;DateStoolSentfromField;DateStoolReceivedNatLevel;DateStoolSentToLab;DateStoolReceivedinLab;FinalCellCultureResult;DateFinalCellCultureResults;FinalITDResult;DateFinalrRTPCRResults;DateIsolateSentforSeq;SequenceName;DateSeqResult;DelaysInProccessingForDDNS;DetailsOfDelays;DateRNAextraction;ExtractionKit;ExtractionType;DateRTPCR;RTPCRMachine;RTPCRcomments;DatePanEVPCR;PanEVPCRMachine;PanEVprimers;PanEVPCRcomments;DateVP1PCR;VP1PCRMachine;VP1primers;VP1PCRcomments;PositiveControlPCRCheck;NegativeControlPCRCheck;LibraryPreparationKit;Well;RunNumber;DateSeqRunLoaded;SequencerUsed;FlowCellVersion;FlowCellID;FlowCellPriorUses;PoresAvilableAtFlowCellCheck;MinKNOWSoftwareVersion;RunHoursDuration;DateFastaGenerated;AnalysisPipelineVersion;RunQC;DDNSclassification;SampleQC;SampleQCChecksComplete;QCComments;ToReport;DateReported;EmergenceGroupVDPV1;EmergenceGroupVDPV2;EmergenceGroupVDPV3;NonPolioEV|closest_reference;NonPolioEV|num_reads;NonPolioEV|nt_diff_from_reference;NonPolioEV|pcent_match;NonPolioEV|classification;Sabin1-related|closest_reference;Sabin1-related|num_reads;Sabin1-related|nt_diff_from_reference;Sabin1-related|pcent_match;Sabin1-related|classification;Sabin2-related|closest_reference;Sabin2-related|num_reads;Sabin2-related|nt_diff_from_reference;Sabin2-related|pcent_match;Sabin2-related|classification;Sabin3-related|closest_reference;Sabin3-related|num_reads;Sabin3-related|nt_diff_from_reference;Sabin3-related|pcent_match;Sabin3-related|classification;WPV1|closest_reference;WPV1|num_reads;WPV1|nt_diff_from_reference;WPV1|pcent_match;WPV1|classification;WPV2|closest_reference;WPV2|num_reads;WPV2|nt_diff_from_reference;WPV2|pcent_match;WPV2|classification;WPV3|closest_reference;WPV3|num_reads;WPV3|nt_diff_from_reference;WPV3|pcent_match;WPV3|classification;PositiveControl|closest_reference;PositiveControl|num_reads;PositiveControl|nt_diff_from_reference;PositiveControl|pcent_match;PositiveControl|classification;comments".split(';')
            merged_df = merged_df[final_fmt]

        except KeyError as e:
                print(e)                
                if self.lang_combobox.currentText() == 'English':
                    QMessageBox.critical(self, 'Error', f'Error originated from merging of files, please check if sample IDs exist in both files!')
            
                elif self.lang_combobox.currentText() == 'Francais':
                    QMessageBox.critical(self, 'Error', f'Les colonnes relatives à sample et/ou barcode ne sont pas identiques entre le rapport Piranha et le fichier LabInfo.')
                
                else:
                    QMessageBox.critical(self, 'Error', f'O erro ocorreu quando os ficheiros foram fundidos. Verifique se as IDs das amostras existem em ambos os ficheiros!')

                # Reset
                self.destination_entry.clear()
                self.epi_entry.clear()
                self.lab_entry.clear()
                self.pir_entry.clear()
                self.runnumber_textbox.clear()
                return

        # save merged sorted dataframe to a new csv file called barcode
        try:
            merged_df.to_csv(f'{destination_path}/{self.runnumber_textbox.text()}_detailed_run_report.csv', index=False, encoding='utf-8')
            
            # Display Success message and clear list
            if self.lang_combobox.currentText() == 'English':
                QMessageBox.information(self, 'Success', f'Files merged correctly')
                
            elif self.lang_combobox.currentText() == 'Francais':
                QMessageBox.information(self, 'Succès', 'CSVs se sont rejoints correctement')
                    
            else:
                QMessageBox.information(self, 'Sucesso', f'Ficheiros fundidos corretamente')

            # Once complete, resets to initial settings
            self.destination_entry.clear()
            self.epi_entry.clear()
            self.lab_entry.clear()
            self.pir_entry.clear()
            self.runnumber_textbox.clear()
            
        except PermissionError:
            if self.lang_combobox.currentText() == 'English':
                QMessageBox.critical(self, 'Permission Error', f'The file couldn\'t be saved. Perhaps a file with the same name is open or the destination is a restricted folder.')
            
            elif self.lang_combobox.currentText() == 'Francais':
                QMessageBox.critical(self, 'Permission Error', f'Le fichier n\'a pas pu être enregistré. Il se peut qu\'un fichier portant le même nom soit ouvert, ou que la destination soit un dossier restreint.')
                
            else:
                QMessageBox.critical(self, 'Permission Error', f'O ficheiro não pôde ser guardado. Pode estar aberto um ficheiro com o mesmo nome ou o destino é uma pasta restrita.')

            # Reset
            self.destination_entry.clear()
            self.epi_entry.clear()
            self.lab_entry.clear()
            self.pir_entry.clear()
            self.runnumber_textbox.clear()
            return
            

    def clear_list(self):
        self.pir_entry.clear()
        self.runnumber_textbox.clear()
        self.destination_entry.clear()
        self.epi_entry.clear()
        self.lab_entry.clear()


if __name__ == '__main__':
    
    
    # Unexpected error catcher
    def handle_exception(exc_type, exc_value, exc_traceback):
        if issubclass(exc_type, KeyboardInterrupt):
            sys.__excepthook__(exc_type, exc_value, exc_traceback)
            return
        
        # exception and traceback
        error_message = "".join(traceback.format_exception(exc_type, exc_value, exc_traceback))
        
        # seting window message with 500 tabs to make it longer
        msg_box = QMessageBox()
        msg_box.setIcon(QMessageBox.Critical)
        msg_box.setWindowTitle("An error occurred" + "  " * 500)
        msg_box.setText(f"An unexpected error occurred:\n{str(exc_value)}\nLog saved to Downloads Folder!")

        # create a scrollable traceback
        scroll = QScrollArea(msg_box)
        scroll.setWidgetResizable(True)
        scroll_content = QWidget()
        scroll_layout = QVBoxLayout(scroll_content)
        
        # Add the traceback message in a QLabel and set it inside the scroll area
        traceback_label = QLabel(error_message)
        traceback_label.setWordWrap(True)
        traceback_label.setStyleSheet("QLabel { font-size: 35px; }")  # Adjust font size here
        scroll_layout.addWidget(traceback_label)
        
        scroll.setWidget(scroll_content)
        
        # adding scrollable traceback 
        msg_box.layout().addWidget(scroll, 1, 0, 1, msg_box.layout().columnCount())
        
        msg_box.exec_()
        
        # Create a unique filename with the current date and time
        timestamp = datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
        log_filename = f"merger_app_error_log_{timestamp}.txt"
        downloads_folder = os.path.join(os.path.expanduser("~"), "Downloads")
        log_filepath = os.path.join(downloads_folder, log_filename)
        
        # Write the error message to the log file on the desktop
        with open(log_filepath, "w") as f:
            f.write(error_message)
            
    # Set excepthook to custom one
    sys.excepthook = handle_exception
    app = QApplication(sys.argv)
    prog = App()
    prog.show()
    sys.exit(app.exec())
