import os, logging, sys
from logging.handlers import RotatingFileHandler
import pandas as pd

from PyQt5.QtCore import Qt
from PyQt5.QtGui import QPixmap, QFont, QIcon
from PyQt5.QtWidgets import QApplication, QMainWindow, QListWidget, QLineEdit, QPushButton, QMessageBox, QLabel, QFileDialog, QComboBox


# AUTHOR - Shean Mobed
# ORG - Biosurv International


# DESCRIPTION
"""
This App takes the Lab Info and EpiInfo files and combines a barcodes.csv file for Piranha pipeline input. The allows the user to input single value lab information into the barcodes.csv file,
this will copy for every sample line in the sheet. This will achieve better data integrity and consistent completion of reports.
"""

# COMPILE COMMAND
"""
CLI MAC OS
python3.11 -m nuitka --macos-app-icon=Icon.icns --include-data-files=Logo.png=Logo.png --include-data-files=Icon.ico=Icon.icns --macos-app-mode=gui --disable-console --macos-create-app-bundle --macos-app-name="CSV Merger App" --onefile --enable-plugin=pyqt5 Merger_before_piranha.py

WINDOWS - in commandprompt
nuitka --onefile --enable-plugins=pyqt5 --include-data-files=Logo.png=./Logo.png --include-data-files=Icon.ico=./Icon.ico --disable-console --windows-icon-from-ico=Icon.ico --company-name="Biosurv International" --product-name="CSV Merger Application" --file-version=2.1.0 --file-description=="This App merges LabID and EpiID files to a standard output"  Merger_before_piranha.py
"""

# CHANGELOG
""""
V2.0.2 --> V2.1.0

- Went back to merging data before Piranha, prior version took piranha report plus lab and epi info for inputs
- Now has textbox entries for single value labinfo to speed up completion, and standardise Primer naming.
- Output file now is prefixed with run number
- universal error handler + log file outputted at chosen destination
- Template generation built into app, just DDNS version for now.
"""

version = "2.1.0"

def setup_logging(log_path=None):
    """Set up logging with a rotating file handler."""
    if log_path is None:
        log_path = os.path.join(os.getcwd(), 'merger_app_error.log')
    else:
        log_path = os.path.join(log_path, 'merger_app_error.log')
    
    try:
        handler = RotatingFileHandler(log_path, maxBytes=1000000, backupCount=5)
        handler.setFormatter(logging.Formatter('%(asctime)s - %(levelname)s - %(message)s'))
        logger = logging.getLogger('')
        logger.setLevel(logging.ERROR)
        logger.handlers = []  # Clear existing handlers
        logger.addHandler(handler)
        return log_path
    except PermissionError:
        logging.error(f"Permission denied writing log to {log_path}", exc_info=True)
        return os.path.join(os.getcwd(), 'merger_app_error.log')

# Initial logging setup
current_log_path = setup_logging()
logging.error(f"Starting Merger version {version}")

class ErrorHandler:
    """Redirect stderr to log errors."""
    def __init__(self):
        self.original_stderr = sys.stderr

    def write(self, message):
        self.original_stderr.write(message)
        if message.strip():
            logging.error(message.strip())

    def flush(self):
        self.original_stderr.flush()

def exception_hook(exctype, value, traceback):
    """Custom exception hook to log unhandled exceptions."""
    logging.error('Unhandled exception', exc_info=(exctype, value, traceback))
    app = QApplication.instance()
    msg = CustomMessageBox("warning", "An unhandled error occurred. Please check the log file (merger_app_error.log) in the output destination for details.")
    msg.exec_()
    sys.__excepthook__(exctype, value, traceback)
    
class CustomMessageBox(QMessageBox):
    """Custom styled message box for warnings and information."""
    def __init__(self, type, message):
        super().__init__()
        self.setWindowTitle(type.capitalize())
        self.setText(message)
        self.setStandardButtons(QMessageBox.Ok)
        warning_style = """
            QMessageBox {background-color: white;color: black;}
            QMessageBox QLabel {color: black;}
            QMessageBox QPushButton {background-color: #ff9800; color: white;padding: 5px;border-radius: 3px; min-width: 80px; margin-left: auto; margin-right: auto;}
            QMessageBox QPushButton:hover {background-color: #e68900;}
        """
        information_style = """
            QMessageBox {background-color: white; color: black;}
            QMessageBox QLabel {color: black;}
            QMessageBox QPushButton {background-color: #2196F3; color: white;padding: 5px;border-radius: 3px; min-width: 80px; margin-left: auto; margin-right: auto;}
            QMessageBox QPushButton:hover {background-color: #1e87d9;}
        """
        self.setStyleSheet(warning_style if type.lower() == "warning" else information_style)


class ListBoxWidget(QListWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setAcceptDrops(True)
        self.resize(500, 500)

    def dragEnterEvent(self, event):
        bg_label.hide()
        if event.mimeData().hasUrls():
            urls = event.mimeData().urls()
            if all(url.toString().endswith('.csv') for url in urls) and len(urls) <= 2:
                event.accept()
            else:
                event.ignore()
        else:
            event.ignore()

    def dragMoveEvent(self, event):
        if event.mimeData().hasUrls():
            event.setDropAction(Qt.CopyAction)
            event.accept()
        else:
            event.ignore()

    def dropEvent(self, event):
        #self.clear()
        urls = event.mimeData().urls()
        file_paths = [url.toLocalFile() for url in urls if url.toString().endswith('.csv')]
        self.addItems(file_paths)

        bg_label.setText('')


class App(QMainWindow):
    def __init__(self):
        super().__init__()

        # ---- APP WINDOW ----
        screen_size = self.screen().size()
        screen_width = int(screen_size.width() * 0.6)
        screen_height = int(screen_size.height() * 0.6)
        self.resize(screen_width, screen_height)
        self.setStyleSheet("background-color: white; color: black;")  # Set window color to white
        self.setWindowIcon(QIcon(os.path.join(os.path.dirname(__file__), "Icon.ico")))  # Sets top left icon to custom icon
        self.setWindowTitle("CSV Merger App")  # App Title
        
        # ---- VERSION ----
        self.version_label = QLabel(f'Version: {version}', self)
        self.version_label.setStyleSheet("background-color:transparent")
        self.version_label.setGeometry(int(screen_width * 0.9), int(screen_height * 0.97), int(screen_width * 0.2), int(screen_height * 0.02))
        self.version_label.setFont(QFont('Arial', 9))

        # ---- LANGUAGE ----
        self.en_btn = QPushButton('EN', self)
        self.en_btn.setGeometry(int(screen_width * 0.88), int(screen_height * 0.02), int(screen_width * 0.05),
                                int(screen_height * 0.05))
        self.en_btn.setStyleSheet("QPushButton{color: white; border-radius:15px; background-color:#2e3192;border-color:black;border-style: solid;border-width: 1px;} QPushButton::pressed{background-color : #3638d8;}")
        self.en_btn.setFont(QFont('Arial', 11))
        self.en_btn.clicked.connect(self.to_eng)

        self.fr_btn = QPushButton('FR', self)
        self.fr_btn.setGeometry(int(screen_width * 0.94), int(screen_height * 0.02), int(screen_width * 0.05),
                                int(screen_height * 0.05))
        self.fr_btn.setStyleSheet("QPushButton{color: white; border-radius:15px; background-color:#2e3192;border-color:black;border-style: solid;border-width: 1px;} QPushButton::pressed{background-color : #3638d8;}")
        self.fr_btn.setFont(QFont('Arial', 11))
        self.fr_btn.clicked.connect(self.to_fr)

        # ---- LOGO ----
        self.logo_label = QLabel(self)
        self.logo_label.setGeometry(int(screen_width * 0.02), int(screen_height * 0.02), int(screen_width * 0.23), int(screen_height * 0.09))  # Logo position and dimension 100, 10, 900, 250
        pixmap = QPixmap(os.path.join(os.path.dirname(__file__), 'Logo.png'))
        self.logo_label.setPixmap(pixmap)
        self.logo_label.setScaledContents(True)

        # ---- DROPBOX ----
        self.listbox_view = ListBoxWidget(self)
        self.listbox_view.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style:dashed;border-width: 2px;border-radius: 10px;")

        self.listbox_view.setGeometry(int(screen_width * 0.12), int(screen_height * 0.11), int(screen_width * 0.63), int(screen_height * 0.25))
        self.listbox_label = QLabel('Dropbox:', self)
        self.listbox_label.setStyleSheet("background-color:transparent")
        self.listbox_label.setGeometry(int(screen_width * 0.02), int(screen_height * 0.21), int(screen_width * 0.2), int(screen_height * 0.05))  
        self.listbox_label.setFont(QFont('Arial', 9))

        global bg_label # set to global scope so it can be changed in various other app functions
        bg_label = QLabel('Drop CSVs here', self)
        bg_label.setStyleSheet("background-color:#FAF9F6")
        bg_label.setGeometry(int(screen_width * 0.35), int(screen_height * 0.21), int(screen_width * 0.2), int(screen_height * 0.05))
        
        # ---- BUTTONS ----
        self.btn_concat = QPushButton('Merge CSVs', self)
        self.btn_concat.setGeometry(int(screen_width * 0.8), int(screen_height * 0.64), int(screen_width * 0.15), int(screen_height * 0.08))  # button position and dimension
        self.btn_concat.setFont(QFont('Arial', 9))
        self.btn_concat.setStyleSheet("QPushButton{color: white; border-radius: 15px;background-color:#2e3192;border-color:black;border-style: solid;border-width: 1px;} QPushButton::pressed{background-color : #3638d8;}")
        self.btn_concat.clicked.connect(self.concatenate_csv)

        self.btn_clear = QPushButton('Clear', self)
        self.btn_clear.setGeometry(int(screen_width * 0.8), int(screen_height * 0.74), int(screen_width * 0.15), int(screen_height * 0.08))  # button position and dimension
        self.btn_clear.setFont(QFont('Arial', 9))
        self.btn_clear.setStyleSheet("QPushButton{color: white; border-radius: 15px;background-color:#2e3192;border-color:black;border-style: solid;border-width: 1px;} QPushButton::pressed{background-color : #3638d8;}")
        self.btn_clear.clicked.connect(self.clear_list)
        
        ## TEMPLATE
        self.btn_template = QPushButton('Generate Template', self)
        self.btn_template.setGeometry(int(screen_width * 0.8), int(screen_height * 0.84), int(screen_width * 0.15), int(screen_height * 0.08))
        self.btn_template.setFont(QFont('Arial', 8))
        self.btn_template.setStyleSheet("QPushButton{color: white; border-radius: 15px;background-color:#2e3192;border-color:black;border-style: solid;border-width: 1px;} QPushButton::pressed{background-color : #3638d8;}")
        self.btn_template.clicked.connect(self.generate_template)

        # User defined file destination
        self.destination_btn = QPushButton('Select', self)
        self.destination_btn.setGeometry(int(screen_width * 0.76), int(screen_height * 0.54), int(screen_width * 0.08), int(screen_height * 0.06))  
        self.destination_btn.setFont(QFont('Arial', 7))
        self.destination_btn.setStyleSheet("QPushButton{color: white; border-radius: 15px;background-color:#2e3192;border-color:black;border-style: solid;border-width: 1px;} QPushButton::pressed{background-color : #3638d8;}")
        self.destination_btn.clicked.connect(lambda: self.select_destination(3))

        self.lab_selec_btn = QPushButton('Select', self)
        self.lab_selec_btn.setGeometry(int(screen_width * 0.76), int(screen_height * 0.47), int(screen_width * 0.08), int(screen_height * 0.06))  
        self.lab_selec_btn.setFont(QFont('Arial', 7))
        self.lab_selec_btn.setStyleSheet("QPushButton{color: white; border-radius: 15px;background-color:#2e3192;border-color:black;border-style: solid;border-width: 1px;} QPushButton::pressed{background-color : #3638d8;}")
        self.lab_selec_btn.clicked.connect(lambda: self.select_destination(2))

        self.epi_selec_btn = QPushButton('Select', self)
        self.epi_selec_btn.setGeometry(int(screen_width * 0.76), int(screen_height * 0.4), int(screen_width * 0.08), int(screen_height * 0.06))  
        self.epi_selec_btn.setFont(QFont('Arial', 7))
        self.epi_selec_btn.setStyleSheet("QPushButton{color: white; border-radius: 15px;background-color:#2e3192;border-color:black;border-style: solid;border-width: 1px;} QPushButton::pressed{background-color : #3638d8;}")
        self.epi_selec_btn.clicked.connect(lambda: self.select_destination(1))

        # ---- DESTINATION BOX ----
        self.destination_entry = QLineEdit(self)
        self.destination_entry.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style: dashed;border-width: 2px;border-radius: 10px;")
        self.destination_entry.setGeometry(int(screen_width * 0.12), int(screen_height * 0.54), int(screen_width * 0.63), int(screen_height * 0.06))
        self.destination_entry.setFont(QFont('Arial', 11))

        self.destination_label = QLabel('Destination:', self)
        self.destination_label.setStyleSheet("background-color:transparent")
        self.destination_label.setGeometry(int(screen_width * 0.02), int(screen_height * 0.54), int(screen_width * 0.2), int(screen_height * 0.05))  
        self.destination_label.setFont(QFont('Arial', 9))

        # ---- EPIINFO BOX ----
        self.epi_entry = QLineEdit(self)
        self.epi_entry.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style: dashed;border-width: 2px;border-radius: 10px;")
        self.epi_entry.setGeometry(int(screen_width * 0.12), int(screen_height * 0.4), int(screen_width * 0.63), int(screen_height * 0.06))
        self.epi_entry.setFont(QFont('Arial', 11))

        self.epi_label = QLabel('Epi Info:', self)
        self.epi_label.setStyleSheet("background-color:transparent")
        self.epi_label.setGeometry(int(screen_width * 0.02), int(screen_height * 0.4), int(screen_width * 0.2), int(screen_height * 0.05))  
        self.epi_label.setFont(QFont('Arial', 9))

        # ---- LABINFO BOX ----
        self.lab_entry = QLineEdit(self)
        self.lab_entry.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style: dashed;border-width: 2px;border-radius: 10px;")
        self.lab_entry.setGeometry(int(screen_width * 0.12), int(screen_height * 0.47), int(screen_width * 0.63), int(screen_height * 0.06))
        self.lab_entry.setFont(QFont('Arial', 11))

        self.lab_label = QLabel('Lab Info:', self)
        self.lab_label.setStyleSheet("background-color:transparent")
        self.lab_label.setGeometry(int(screen_width * 0.02), int(screen_height * 0.47), int(screen_width * 0.2), int(screen_height * 0.05))  
        self.lab_label.setFont(QFont('Arial', 9))
        
        # ---- SINGLE VALUES ----
        self.run_entry = QLineEdit(self)
        self.run_entry.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style: dashed;border-width: 2px;border-radius: 10px;")
        self.run_entry.setGeometry(int(screen_width * 0.12), int(screen_height * 0.61), int(screen_width * 0.16), int(screen_height * 0.06))
        self.run_entry.setFont(QFont('Arial', 11))

        self.run_label = QLabel('Run Number:', self)
        self.run_label.setStyleSheet("background-color:transparent")
        self.run_label.setGeometry(int(screen_width * 0.02), int(screen_height * 0.61), int(screen_width * 0.2), int(screen_height * 0.05))  
        self.run_label.setFont(QFont('Arial', 9))
        
        self.date_seq_entry = QLineEdit(self)
        self.date_seq_entry.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style: dashed;border-width: 2px;border-radius: 10px;")
        self.date_seq_entry.setGeometry(int(screen_width * 0.41), int(screen_height * 0.61), int(screen_width * 0.1), int(screen_height * 0.06))
        self.date_seq_entry.setFont(QFont('Arial', 11))

        self.date_seq_label = QLabel('Date Sequenced:', self)
        self.date_seq_label.setStyleSheet("background-color:transparent")
        self.date_seq_label.setGeometry(int(screen_width * 0.29), int(screen_height * 0.61), int(screen_width * 0.12), int(screen_height * 0.05))  
        self.date_seq_label.setFont(QFont('Arial', 9))
        
        self.seq_mach_entry = QLineEdit(self)
        self.seq_mach_entry.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style: dashed;border-width: 2px;border-radius: 10px;")
        self.seq_mach_entry.setGeometry(int(screen_width * 0.59), int(screen_height * 0.61), int(screen_width * 0.16), int(screen_height * 0.06))
        self.seq_mach_entry.setFont(QFont('Arial', 11))

        self.seq_mach_label = QLabel('Sequencer:', self)
        self.seq_mach_label.setStyleSheet("background-color:transparent")
        self.seq_mach_label.setGeometry(int(screen_width * 0.513), int(screen_height * 0.61), int(screen_width * 0.1), int(screen_height * 0.05))  
        self.seq_mach_label.setFont(QFont('Arial', 9))
        
        self.fc_id_entry = QLineEdit(self)
        self.fc_id_entry.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style: dashed;border-width: 2px;border-radius: 10px;")
        self.fc_id_entry.setGeometry(int(screen_width * 0.12), int(screen_height * 0.68), int(screen_width * 0.1), int(screen_height * 0.06))
        self.fc_id_entry.setFont(QFont('Arial', 11))

        self.fc_id_label = QLabel('FlowCell ID:', self)
        self.fc_id_label.setStyleSheet("background-color:transparent")
        self.fc_id_label.setGeometry(int(screen_width * 0.02), int(screen_height * 0.68), int(screen_width * 0.1), int(screen_height * 0.05))  
        self.fc_id_label.setFont(QFont('Arial', 9))
        
        self.fc_ver_entry = QLineEdit(self)
        self.fc_ver_entry.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style: dashed;border-width: 2px;border-radius: 10px;")
        self.fc_ver_entry.setGeometry(int(screen_width * 0.31), int(screen_height * 0.68), int(screen_width * 0.11), int(screen_height * 0.06))
        self.fc_ver_entry.setFont(QFont('Arial', 11))

        self.fc_ver_label = QLabel('FC Version:', self)
        self.fc_ver_label.setStyleSheet("background-color:transparent")
        self.fc_ver_label.setGeometry(int(screen_width * 0.23), int(screen_height * 0.68), int(screen_width * 0.08), int(screen_height * 0.05))  
        self.fc_ver_label.setFont(QFont('Arial', 9))
        
        self.fc_uses_entry = QLineEdit(self)
        self.fc_uses_entry.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style: dashed;border-width: 2px;border-radius: 10px;")
        self.fc_uses_entry.setGeometry(int(screen_width * 0.53), int(screen_height * 0.68), int(screen_width * 0.025), int(screen_height * 0.06))
        self.fc_uses_entry.setFont(QFont('Arial', 11))

        self.fc_uses_label = QLabel('FC Prior uses:', self)
        self.fc_uses_label.setStyleSheet("background-color:transparent")
        self.fc_uses_label.setGeometry(int(screen_width * 0.43), int(screen_height * 0.68), int(screen_width * 0.1), int(screen_height * 0.05))  
        self.fc_uses_label.setFont(QFont('Arial', 9))
        
        self.fc_pores_entry = QLineEdit(self)
        self.fc_pores_entry.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style: dashed;border-width: 2px;border-radius: 10px;")
        self.fc_pores_entry.setGeometry(int(screen_width * 0.69), int(screen_height * 0.68), int(screen_width * 0.06), int(screen_height * 0.06))
        self.fc_pores_entry.setFont(QFont('Arial', 11))

        self.fc_pores_label = QLabel('FC Check Pores:', self)
        self.fc_pores_label.setStyleSheet("background-color:transparent")
        self.fc_pores_label.setGeometry(int(screen_width * 0.56), int(screen_height * 0.68), int(screen_width * 0.12), int(screen_height * 0.05))  
        self.fc_pores_label.setFont(QFont('Arial', 9))

        self.run_time_entry = QLineEdit(self)
        self.run_time_entry.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style: dashed;border-width: 2px;border-radius: 10px;")
        self.run_time_entry.setGeometry(int(screen_width * 0.12), int(screen_height * 0.75), int(screen_width * 0.025), int(screen_height * 0.06))
        self.run_time_entry.setFont(QFont('Arial', 11))

        self.run_time_label = QLabel('Run Hours:', self)
        self.run_time_label.setStyleSheet("background-color:transparent")
        self.run_time_label.setGeometry(int(screen_width * 0.02), int(screen_height * 0.75), int(screen_width * 0.1), int(screen_height * 0.05))  
        self.run_time_label.setFont(QFont('Arial', 9))
        
        self.fasta_gen_entry = QLineEdit(self)
        self.fasta_gen_entry.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style: dashed;border-width: 2px;border-radius: 10px;")
        self.fasta_gen_entry.setGeometry(int(screen_width * 0.29), int(screen_height * 0.75), int(screen_width * 0.1), int(screen_height * 0.06))
        self.fasta_gen_entry.setFont(QFont('Arial', 11))

        self.fasta_gen_label = QLabel('Date FASTA created:', self)
        self.fasta_gen_label.setStyleSheet("background-color:transparent")
        self.fasta_gen_label.setGeometry(int(screen_width * 0.15), int(screen_height * 0.75), int(screen_width * 0.135), int(screen_height * 0.05))  
        self.fasta_gen_label.setFont(QFont('Arial', 9))
        
        self.minknow_ver_entry = QLineEdit(self)
        self.minknow_ver_entry.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style: dashed;border-width: 2px;border-radius: 10px;")
        self.minknow_ver_entry.setGeometry(int(screen_width * 0.525), int(screen_height * 0.75), int(screen_width * 0.07), int(screen_height * 0.06))
        self.minknow_ver_entry.setFont(QFont('Arial', 11))

        self.minknow_ver_label = QLabel('MinKNOW Version:', self)
        self.minknow_ver_label.setStyleSheet("background-color:transparent")
        self.minknow_ver_label.setGeometry(int(screen_width * 0.395), int(screen_height * 0.75), int(screen_width * 0.135), int(screen_height * 0.05))  
        self.minknow_ver_label.setFont(QFont('Arial', 9))
        
        self.piranha_ver_entry = QLineEdit(self)
        self.piranha_ver_entry.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style: dashed;border-width: 2px;border-radius: 10px;")
        self.piranha_ver_entry.setGeometry(int(screen_width * 0.705), int(screen_height * 0.75), int(screen_width * 0.045), int(screen_height * 0.06))
        self.piranha_ver_entry.setFont(QFont('Arial', 11))

        self.piranha_ver_label = QLabel('Piranha Version:', self)
        self.piranha_ver_label.setStyleSheet("background-color:transparent")
        self.piranha_ver_label.setGeometry(int(screen_width * 0.596), int(screen_height * 0.75), int(screen_width * 0.105), int(screen_height * 0.05))  
        self.piranha_ver_label.setFont(QFont('Arial', 9))
        
        self.rtpcr_mach_entry = QLineEdit(self)
        self.rtpcr_mach_entry.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style: dashed;border-width: 2px;border-radius: 10px;")
        self.rtpcr_mach_entry.setGeometry(int(screen_width * 0.12), int(screen_height * 0.82), int(screen_width * 0.3), int(screen_height * 0.06))
        self.rtpcr_mach_entry.setFont(QFont('Arial', 11))

        self.rtpcr_mach_label = QLabel('rtPCR Machine:', self)
        self.rtpcr_mach_label.setStyleSheet("background-color:transparent")
        self.rtpcr_mach_label.setGeometry(int(screen_width * 0.02), int(screen_height * 0.82), int(screen_width * 0.1), int(screen_height * 0.05))  
        self.rtpcr_mach_label.setFont(QFont('Arial', 9))
        
        self.rtpcr_primers_entry = QComboBox(self)
        self.rtpcr_primers_entry.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style: dashed;border-width: 2px;border-radius: 10px;")
        self.rtpcr_primers_entry.setGeometry(int(screen_width * 0.53), int(screen_height * 0.82), int(screen_width * 0.22), int(screen_height * 0.06))
        self.rtpcr_primers_entry.setFont(QFont('Arial', 11))
        
        # primer options
        self.rtpcr_primers_entry.addItems(["Y7+Cre+nOPV2-mm","5'NTR+Cre+nOPV2-mm","5'NTR+Cre"])
        
        self.rtpcr_primers_label = QLabel('rtPCR Primers:', self)
        self.rtpcr_primers_label.setStyleSheet("background-color:transparent")
        self.rtpcr_primers_label.setGeometry(int(screen_width * 0.425), int(screen_height * 0.82), int(screen_width * 0.1), int(screen_height * 0.05))  
        self.rtpcr_primers_label.setFont(QFont('Arial', 9))
        
        self.vp1_mach_entry = QLineEdit(self)
        self.vp1_mach_entry.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style: dashed;border-width: 2px;border-radius: 10px;")
        self.vp1_mach_entry.setGeometry(int(screen_width * 0.12), int(screen_height * 0.89), int(screen_width * 0.3), int(screen_height * 0.06))
        self.vp1_mach_entry.setFont(QFont('Arial', 11))

        self.vp1_mach_label = QLabel('VP1 Machine:', self)
        self.vp1_mach_label.setStyleSheet("background-color:transparent")
        self.vp1_mach_label.setGeometry(int(screen_width * 0.02), int(screen_height * 0.89), int(screen_width * 0.1), int(screen_height * 0.05))  
        self.vp1_mach_label.setFont(QFont('Arial', 9))
        
        self.vp1_primers_entry = QComboBox(self)
        self.vp1_primers_entry.setStyleSheet("background-color:#FAF9F6;border-color:lightgrey;border-style: dashed;border-width: 2px;border-radius: 10px;")
        self.vp1_primers_entry.setGeometry(int(screen_width * 0.53), int(screen_height * 0.89), int(screen_width * 0.22), int(screen_height * 0.06))
        self.vp1_primers_entry.setFont(QFont('Arial', 11))
        
        self.vp1_primers_entry.addItems(["Y7+Q8"]) # primer options
        
        self.vp1_primers_label = QLabel('VP1 Primers:', self)
        self.vp1_primers_label.setStyleSheet("background-color:transparent")
        self.vp1_primers_label.setGeometry(int(screen_width * 0.425), int(screen_height * 0.89), int(screen_width * 0.1), int(screen_height * 0.05))  
        self.vp1_primers_label.setFont(QFont('Arial', 9))
        
    def generate_template(self):
        
        # Saving template to downloads
        downloads_folder = os.path.join(os.path.expanduser("~"), "Downloads")
            
        final_format = 'sample,barcode,IsQCRetest,IfRetestOriginalRun,EPID,SampleType,CaseOrContact,Country,Province,District,StoolCondition,SpecimenNumber,DateOfOnset,DateStoolCollected,DateStoolReceivedinLab,DateStoolsuspension,DateRNAextraction,DateRTPCR,RTPCRMachine,RTPCRprimers,DateVP1PCR,VP1PCRMachine,VP1primers,PositiveControlPCRCheck,NegativeControlPCRCheck,LibraryPreparationKit,Well,RunNumber,DateSeqRunLoaded,SequencerUsed,FlowCellVersion,FlowCellID,FlowCellPriorUses,PoresAvilableAtFlowCellCheck,MinKNOWSoftwareVersion,RunHoursDuration,DateFastaGenerated,AnalysisPipelineVersion,RunQC,DDNSclassification,SampleQC,SampleQCChecksComplete,QCComments,DateReported'.split(',')
        labinfo_template = pd.DataFrame(columns=final_format)
        file_path = os.path.join(downloads_folder, "sample_template.csv")
        labinfo_template.to_csv(file_path, index=False)
        QMessageBox.information(self,'Saved','Sample CSV Template saved to Downloads')


    def to_eng(self):
        # Resets language to English
        self.btn_clear.setText('')
        self.btn_clear.setText('Clear')

        self.btn_concat.setText('')
        self.btn_concat.setText('Merge CSVs')

        self.destination_btn.setText('')
        self.destination_btn.setText('Select')

        self.epi_selec_btn.setText('')
        self.epi_selec_btn.setText('Select')

        self.lab_selec_btn.setText('')
        self.lab_selec_btn.setText('Select')
        
        self.btn_template.setText('')
        self.btn_template.setText('Generate Template')

        bg_label.setText('Drop CSVs Here')

    def to_fr(self):
        # Sets language to French
        self.btn_clear.setText('')
        self.btn_clear.setText('Effacer')

        self.btn_concat.setText('')
        self.btn_concat.setText('Rejoindre CSVs')

        self.destination_btn.setText('')
        self.destination_btn.setText('Sélectionner')

        self.epi_selec_btn.setText('')
        self.epi_selec_btn.setText('Sélectionner')

        self.lab_selec_btn.setText('')
        self.lab_selec_btn.setText('Sélectionner')
        
        self.btn_template.setText('')
        self.btn_template.setText('Générer Template')

        bg_label.setText('Déposez les CSV ici')

    def select_destination(self, type):
        file_dialog = QFileDialog()
        file_dialog.setFileMode(QFileDialog.AnyFile if type in (1, 2) else QFileDialog.Directory)
        
        if file_dialog.exec_():
            selected_files = file_dialog.selectedFiles()
            if selected_files:
                if type == 1:
                    self.epi_entry.setText(selected_files[0])
                    self.listbox_view.addItem(selected_files[0])
                    bg_label.hide()

                elif type == 2:
                    self.lab_entry.setText(selected_files[0])
                    self.listbox_view.addItem(selected_files[0])
                    bg_label.hide()

                elif type == 3:
                    self.destination_entry.setText(selected_files[0])

                    
    def concatenate_csv(self):
        destination_path = self.destination_entry.text()

        if destination_path == '':
            if self.btn_clear.text() == 'Clear':
                QMessageBox.warning(self, 'Warning', 'No destination selected')
                return
            else:
                QMessageBox.waring(self, 'Attention', "Veuillez selectionner une destination")
                return
        

        paths = [self.listbox_view.item(i).text() for i in range(self.listbox_view.count())]
        if len(paths) != 2:
            if self.btn_clear.text() == 'Clear':
                QMessageBox.warning(self, 'Warning', 'Please select exactly two CSV files.')
            else:
                QMessageBox.waring(self, 'Attention', "Veuillez selectionner deux fichier CSV")

            return
     
        input_file1_df = pd.read_csv(paths[0], engine='python',sep=None, encoding='utf-8', encoding_errors='replace')
        input_file2_df = pd.read_csv(paths[1], engine='python',sep=None, encoding='utf-8', encoding_errors='replace')

        #print(input_file1_df.columns)
        #print(input_file2_df.columns)

        if input_file1_df.columns[0].endswith("sample"):
            lablist = input_file1_df
            epiinfo = input_file2_df

        elif input_file2_df.columns[0].endswith("sample"):
            lablist = input_file2_df
            epiinfo = input_file1_df

        else:
            if self.btn_clear.text() == 'Clear':
                QMessageBox.critical(self, 'Error', "Neither file has the correct header!")
            else:
                QMessageBox.critical(self, 'Error', "Les deux CSV n'ont pas le bon format!")

        # downsize columns from epiinfo
        try:
            epi_fmt = ['ICLabID','EpidNumber','CaseOrContact','Country','Province','District','StoolCondition','SpecimenNumber','DateOfOnset','DateStoolCollected','DateStoolReceivedinLab']
            #epiinfo = epiinfo.reindex(epiinfo.columns.union(epi_fmt, sort=False), axis=1, fill_value='')
            
            epiinfo = epiinfo[epi_fmt]

            def encode_col(series):
                 return series.str.normalize('NFKD').str.encode('ascii', errors='ignore').str.decode('utf-8')
            
            epiinfo = epiinfo.astype(str)
            epiinfo = epiinfo.apply(lambda x:encode_col(x))

        
        except KeyError as e:
                missing = str(e).rsplit('\']')[0].split('"[\'')[1]
                epiinfo[missing] = ''
                print(missing)
                
        
                # eng_message = f'Error originated from Epi Info File,\n These essential columns were not found:\n{missing}'
                # fr_message = f'L\'erreur provient du fichier Epi Info,\n Ces colonnes essentielles n\'ont pas été trouvées :\n{missing}'

                # if self.btn_clear.text() == 'Clear':
                #     QMessageBox.critical(self, 'Error', f"{eng_message}")
                
                # else:
                #     QMessageBox.critical(self, 'Error', f"{fr_message}")

                # # Reset
                # self.listbox_view.clear()
                # self.destination_entry.clear()
                # self.epi_entry.clear()
                # self.lab_entry.clear()
                # bg_label.setText('Drop CSVs here')
                # bg_label.show()
                # return
                
        # Fill in single values from merger interface
        lablist['RTPCRMachine'] = self.rtpcr_mach_entry.text()
        lablist['RTPCRprimers'] = self.rtpcr_primers_entry.currentText()
        lablist['VP1Machine'] = self.vp1_mach_entry.text()
        lablist['VP1primers'] = self.vp1_primers_entry.currentText()
        lablist['RunNumber'] = self.run_entry.text()
        lablist['DateSeqRunLoaded'] = self.date_seq_entry.text() 
        lablist['SequencerUsed'] = self.seq_mach_entry.text()
        lablist['FlowCellVersion'] = self.fc_ver_entry.text()
        lablist['FlowCellID'] = self.fc_id_entry.text()
        lablist['FlowCellPriorUses'] = self.fc_uses_entry.text()
        lablist['PoresAvilableAtFlowCellCheck'] = self.fc_pores_entry.text()
        lablist['MinKNOWSoftwareVersion'] = self.minknow_ver_entry.text()
        lablist['RunHoursDuration'] = self.run_time_entry.text()
        lablist['DateFastaGenerated'] = self.fasta_gen_entry.text()
        lablist['AnalysisPipelineVersion'] = self.piranha_ver_entry.text()


        try:
            epiinfo = epiinfo.rename(columns={'ICLabID':'sample','EpidNumber':'EPID'})
            
            # merge dataframes based on sample from lablist and ICLabID from epiinfo
            lablist.columns = lablist.columns.str.encode('ascii','ignore').str.decode('ascii')
            #print(lablist.columns)
            epiinfo['sample'] = epiinfo['sample'].astype("object")
            lablist['sample'] = lablist['sample'].astype("object")

        except KeyError as e:
                eng_message = f'Error originated from lab info file, please check if correct format,or file, was chosen!'
                fr_message = f'L\'erreur provient du fichier Lab Info, veuillez vérifier si le format, ou le fichier correct, a été choisi !'

                if self.btn_clear.text() == 'Clear':
                    QMessageBox.critical(self, 'Error', f"{eng_message}")
            
                else:
                    QMessageBox.critical(self, 'Error', f"{fr_message}")

                # Reset
                self.listbox_view.clear()
                self.destination_entry.clear()
                self.epi_entry.clear()
                self.lab_entry.clear()
                bg_label.setText('Drop CSVs here')
                bg_label.show()
                return
        try:
            lablist['key'] = 0
            epiinfo['key'] = 0
            #merged_df = pd.concat([lablist, epiinfo], axis=1)
            # print(lablist.head())
            # print(epiinfo.head())
            epi_drop = ['EPID','CaseOrContact','Country','Province','District','StoolCondition','SpecimenNumber','DateOfOnset','DateStoolCollected','DateStoolReceivedinLab']
            lablist.drop(columns=epi_drop,inplace=True)
            merged_df = lablist.merge(epiinfo, how='left', on='sample')
            print(merged_df.head())
            # reorder columns
        
            # Selecting specific columns and reordering columns from labinfo and epiinfo
            final_format = 'sample,barcode,IsQCRetest,IfRetestOriginalRun,EPID,SampleType,CaseOrContact,Country,Province,District,StoolCondition,SpecimenNumber,DateOfOnset,DateStoolCollected,DateStoolReceivedinLab,DateStoolsuspension,DateRNAextraction,DateRTPCR,RTPCRMachine,RTPCRprimers,DateVP1PCR,VP1PCRMachine,VP1primers,PositiveControlPCRCheck,NegativeControlPCRCheck,LibraryPreparationKit,Well,RunNumber,DateSeqRunLoaded,SequencerUsed,FlowCellVersion,FlowCellID,FlowCellPriorUses,PoresAvilableAtFlowCellCheck,MinKNOWSoftwareVersion,RunHoursDuration,DateFastaGenerated,AnalysisPipelineVersion,RunQC,DDNSclassification,SampleQC,SampleQCChecksComplete,QCComments,DateReported'.split(',')
            merged_df = merged_df[final_format]

        except KeyError as e:
                print('Key error during merge')
                print(e)
                eng_message = f'Error originated from merging of both files, please check if sample IDs exist in both files!'
                fr_message = f'L\'erreur provient de la fusion des deux fichiers, veuillez vérifier si les IDs existent dans les deux fichiers !'

                if self.btn_clear.text() == 'Clear':
                    QMessageBox.critical(self, 'Error', f"{eng_message}")
                
                else:
                    QMessageBox.critical(self, 'Error', f"{fr_message}")

                # Reset
                self.listbox_view.clear()
                self.destination_entry.clear()
                self.epi_entry.clear()
                self.lab_entry.clear()
                bg_label.setText('Drop CSVs here')
                bg_label.show()
                return

        # save merged sorted dataframe to a new csv file called barcode
        merged_df.to_csv(f'{destination_path}/{self.run_entry.text()}_barcodes.csv', index=False, encoding='utf-8')

        # Display Success message and clear list
        if self.btn_clear.text() == 'Clear':
            QMessageBox.information(self, 'Success', f'Files merged Correctly')
        else:
            QMessageBox.information(self, 'Success', 'CSVs se sont rejoints correctement')

        # Once complete, resets to initial settings
        self.listbox_view.clear()
        self.destination_entry.clear()
        self.epi_entry.clear()
        self.lab_entry.clear()
        bg_label.setText('Drop CSVs here')
        bg_label.show()

    def clear_list(self):
        self.listbox_view.clear()
        self.destination_entry.clear()
        self.epi_entry.clear()
        self.lab_entry.clear()
        
        self.rtpcr_mach_entry.clear()
        self.vp1_mach_entry.clear()
        self.run_entry.clear()
        self.date_seq_entry.clear() 
        self.seq_mach_entry.clear()
        self.fc_ver_entry.clear()
        self.fc_id_entry.clear()
        self.fc_uses_entry.clear()
        self.fc_pores_entry.clear()
        self.minknow_ver_entry.clear()
        self.run_time_entry.clear()
        self.fasta_gen_entry.clear()
        self.piranha_ver_entry.clear()
        
        bg_label.setText('Drop CSVs here')
        bg_label.show()


if __name__ == '__main__':
    try:
        app = QApplication(sys.argv)
        prog = App()
        prog.show()
        sys.exit(app.exec())
    
    except Exception as e:
        logging.error("Application initialization failed", exc_info=True)
        print(f"Application failed to start: {str(e)}")
