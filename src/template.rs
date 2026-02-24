use polars::prelude::*;

pub fn expected_ddns_columns() -> Vec<&'static str> {
    vec![
        "sample", "barcode", "IsQCRetest", "IfRetestOriginalRun", "EPID",
        "SampleType", "CaseOrContact", "Country", "Province", "District", "StoolCondition",
        "SpecimenNumber", "SameAliquot", "DateOfOnset", "DateStoolCollected", "DateStoolReceivedinLab", "DateStoolsuspension",
        "DateRNAextraction", "DateRTPCR", "RTPCRMachine", "RTPCRprimers", "DateVP1PCR", "VP1PCRMachine",
        "VP1primers", "PositiveControlPCRCheck", "NegativeControlPCRCheck",
        "LibraryPreparationKit", "Well", "RunNumber", "DateSeqRunLoaded", "SequencerUsed",
        "FlowCellVersion", "FlowCellID", "FlowCellPriorUses", "PoresAvilableAtFlowCellCheck",
        "MinKNOWSoftwareVersion", "RunHoursDuration", "DateFastaGenerated", "AnalysisPipelineVersion", "RunQC", "DDNSclassification",
        "SampleQC", "SampleQCChecksComplete", "QCComments", "DateReported"
    ]
}

pub fn expected_minion_columns() -> Vec<&'static str> {
    vec![
        "sample", "barcode", "IsQCRetest", "IfRetestOriginalRun", "institute", "EPID", "CaseOrContact", "CountryOfSampleOrigin",
        "SpecimenNumber", "DateOfOnset", "DateStoolCollected", "DateStoolReceivedinLab", "DateStoolsuspension",
        "DateFinalCultureResult", "FlaskNumber",
        "FinalCellCultureResult", "DateFinalITDresult", "ITDResult", "ITDMixture", "DateSangerResultGenerated",
        "SangerSequenceID", "SequencingLab", "DelaysInProcessingForSequencing", "DetailsOfDelays", "IsclassificationQCRetest",
        "RTPCRcomments", "DateRNAExtraction", "DateRTPCR", "PositiveControlPCRCheck", "NegativeControlPCRheck",
        "LibraryPreparationKit", "RunNumber", "DateSeqRunLoaded", "FlowCellID", "FlowCellPriorUses",
        "PoresAvilableAtFlowCellCheck", "MinKNOWSoftwareVersion", "RunHoursDuration", "DateFastaGenerated",
        "AnalysisPipelineVersion", "RunQC", "IsolateClassification", "SampleQC", "SampleQCChecksComplete", "QCComments", "DateReported"
    ]
}

pub fn expected_columns_for_mode(mode: &str) -> Vec<&'static str> {
    if mode == "minION" {
        expected_minion_columns()
    } else {
        expected_ddns_columns()
    }
}

pub fn create_minion_template() -> PolarsResult<DataFrame> {
    df![
        "sample" => Vec::<String>::new(),
        "barcode" => Vec::<String>::new(),
        "IsQCRetest" => Vec::<String>::new(),
        "IfRetestOriginalRun" => Vec::<String>::new(),
        "institute" => Vec::<String>::new(),
        "EPID" => Vec::<String>::new(),
        "CaseOrContact" => Vec::<String>::new(),
        "CountryOfSampleOrigin" => Vec::<String>::new(),
        "SpecimenNumber" => Vec::<String>::new(),
        "DateOfOnset" => Vec::<String>::new(),
        "DateStoolCollected" => Vec::<String>::new(),
        "DateStoolReceivedinLab" => Vec::<String>::new(),
        "DateStoolsuspension" => Vec::<String>::new(),
        "TypeofPositiveControl" => Vec::<String>::new(),
        "DatePositiveControlreconstituted" => Vec::<String>::new(),
        "DateFinalCultureResult" => Vec::<String>::new(),
        "FlaskNumber" => Vec::<String>::new(),
        "FinalCellCultureResult" => Vec::<String>::new(),
        "DateFinalITDresult" => Vec::<String>::new(),
        "ITDResult" => Vec::<String>::new(),
        "ITDMixture" => Vec::<String>::new(),
        "DateSangerResultGenerated" => Vec::<String>::new(),
        "SangerSequenceID" => Vec::<String>::new(),
        "SequencingLab" => Vec::<String>::new(),
        "DelaysInProcessingForSequencing" => Vec::<String>::new(),
        "DetailsOfDelays" => Vec::<String>::new(),
        "IsclassificationQCRetest" => Vec::<String>::new(),
        "RTPCRcomments" => Vec::<String>::new(),
        "DateRNAExtraction" => Vec::<String>::new(),
        "DateRTPCR" => Vec::<String>::new(),
        "PositiveControlPCRCheck" => Vec::<String>::new(),
        "NegativeControlPCRheck" => Vec::<String>::new(),
        "LibraryPreparationKit" => Vec::<String>::new(),
        "RunNumber" => Vec::<String>::new(),
        "DateSeqRunLoaded" => Vec::<String>::new(),
        "FlowCellID" => Vec::<String>::new(),
        "FlowCellPriorUses" => Vec::<String>::new(),
        "PoresAvilableAtFlowCellCheck" => Vec::<String>::new(),
        "MinKNOWSoftwareVersion" => Vec::<String>::new(),
        "RunHoursDuration" => Vec::<String>::new(),
        "DateFastaGenerated" => Vec::<String>::new(),
        "AnalysisPipelineVersion" => Vec::<String>::new(),
        "RunQC" => Vec::<String>::new(),
        "IsolateClassification" => Vec::<String>::new(),
        "SampleQC" => Vec::<String>::new(),
        "SampleQCChecksComplete" => Vec::<String>::new(),
        "QCComments" => Vec::<String>::new(),
        "DateReported" => Vec::<String>::new()
    ]
}

pub fn create_ddns_template() -> PolarsResult<DataFrame> {
    df![
        "sample" => Vec::<String>::new(),
        "barcode" => Vec::<String>::new(),
        "IsQCRetest" => Vec::<String>::new(),
        "IfRetestOriginalRun" => Vec::<String>::new(),
        "EPID" => Vec::<String>::new(),
        "SampleType" => Vec::<String>::new(),
        "CaseOrContact" => Vec::<String>::new(),
        "Country" => Vec::<String>::new(),
        "Province" => Vec::<String>::new(),
        "District" => Vec::<String>::new(),
        "StoolCondition" => Vec::<String>::new(),
        "SpecimenNumber" => Vec::<String>::new(),
        "SameAliquot" => Vec::<String>::new(),
        "DateOfOnset" => Vec::<String>::new(),
        "DateStoolCollected" => Vec::<String>::new(),
        "DateStoolReceivedinLab" => Vec::<String>::new(),
        "DateStoolsuspension" => Vec::<String>::new(),
        "DateRNAextraction" => Vec::<String>::new(),
        "DateRTPCR" => Vec::<String>::new(),
        "RTPCRMachine" => Vec::<String>::new(),
        "RTPCRprimers" => Vec::<String>::new(),
        "DateVP1PCR" => Vec::<String>::new(),
        "VP1PCRMachine" => Vec::<String>::new(),
        "VP1primers" => Vec::<String>::new(),
        "PositiveControlPCRCheck" => Vec::<String>::new(),
        "NegativeControlPCRCheck" => Vec::<String>::new(),
        "LibraryPreparationKit" => Vec::<String>::new(),
        "Well" => Vec::<String>::new(),
        "RunNumber" => Vec::<String>::new(),
        "DateSeqRunLoaded" => Vec::<String>::new(),
        "SequencerUsed" => Vec::<String>::new(),
        "FlowCellVersion" => Vec::<String>::new(),
        "FlowCellID" => Vec::<String>::new(),
        "FlowCellPriorUses" => Vec::<String>::new(),
        "PoresAvilableAtFlowCellCheck" => Vec::<String>::new(),
        "MinKNOWSoftwareVersion" => Vec::<String>::new(),
        "RunHoursDuration" => Vec::<String>::new(),
        "DateFastaGenerated" => Vec::<String>::new(),
        "AnalysisPipelineVersion" => Vec::<String>::new(),
        "RunQC" => Vec::<String>::new(),
        "DDNSclassification" => Vec::<String>::new(),
        "SampleQC" => Vec::<String>::new(),
        "SampleQCChecksComplete" => Vec::<String>::new(),
        "QCComments" => Vec::<String>::new(),
        "DateReported" => Vec::<String>::new()
    ]
}

pub fn create_template_for_mode(mode: &str) -> PolarsResult<DataFrame> {
    if mode == "minION" {
        create_minion_template()
    } else {
        create_ddns_template()
    }
}
