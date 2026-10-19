import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { api } from '../api/client';
import { UploadCloud, Download, FileSpreadsheet, AlertCircle, CheckCircle2, FileText, Sparkles } from 'lucide-react';

export default function UploadPage() {
  const navigate = useNavigate();
  const [file, setFile] = useState<File | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const handleDownloadTemplate = () => {
    const csvContent = api.downloadTemplate();
    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.setAttribute('download', 'aicte_apaar_template_csv-v1.csv');
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };

  const loadSampleDataset = async (type: 'demo' | 'clean' | 'overrun') => {
    let csvData = '';
    let fileName = '';

    if (type === 'clean') {
      fileName = 'clean_batch.csv';
      csvData =
        'apaar_id,institution_student_id,full_name,programme_code,academic_year,admission_date,seat_category,admission_status\n' +
        'APAAR-DEMO-0001,ALPHA-0001,Asha Rao,CSE-01,2026-27,2026-07-01,REGULAR,ACTIVE\n' +
        'APAAR-DEMO-0002,ALPHA-0002,Bejoy Iyer,CSE-01,2026-27,2026-07-03,REGULAR,ACTIVE\n' +
        'APAAR-DEMO-0003,ALPHA-0003,Chitra Menon,ECE-01,2026-27,2026-07-05,REGULAR,ACTIVE\n' +
        'APAAR-DEMO-0004,ALPHA-0004,Deepak Kulkarni,CSE-01,2026-27,2026-07-06,NRI,ACTIVE\n' +
        'APAAR-DEMO-0005,ALPHA-0005,Esha Verma,ECE-01,2026-27,2026-07-08,REGULAR,ENROLLED\n' +
        'APAAR-DEMO-0006,ALPHA-0006,Farhan Ali,CSE-01,2026-27,2026-07-09,OCI,ACTIVE\n';
    } else if (type === 'demo') {
      fileName = 'demo_batch.csv';
      csvData =
        'apaar_id,institution_student_id,full_name,programme_code,academic_year,admission_date,seat_category,admission_status\n' +
        'APAAR-DEMO-0001,ALPHA-0001,Asha Rao,CSE-01,2026-27,2026-07-01,REGULAR,ACTIVE\n' +
        ',ALPHA-0002,Bejoy Iyer,CSE-01,2026-27,2026-07-03,REGULAR,ACTIVE\n' +
        'APAAR-DEMO-0001,ALPHA-0003,Chitra Menon,CSE-01,2026-27,2026-07-05,REGULAR,ACTIVE\n' +
        'APAAR-DEMO-0004,ALPHA-0004,Deepak Kulkarni,CSE-01,2026-27,05-07-2026,REGULAR,ACTIVE\n' +
        'APAAR-DEMO-0005,ALPHA-0005,Esha Verma,UNKNOWN-99,2026-27,2026-07-08,REGULAR,ACTIVE\n' +
        'APAAR-DEMO-0006,ALPHA-0006,,CSE-01,2026-27,2026-07-09,REGULAR,ACTIVE\n' +
        'APAAR-DEMO-0007,ALPHA-0007,Ganesh Iyer,CSE-01,2026-27,2026-07-10,SCHOLARSHIP,ACTIVE\n' +
        'APAAR-DEMO-0008,ALPHA-0008,Harini Nair,CSE-01,2026-27,2026-07-11,REGULAR,ACTIVE\n';
    } else {
      fileName = 'intake_overrun.csv';
      // 65 regular admissions against 60 regular intake capacity
      const lines = [
        'apaar_id,institution_student_id,full_name,programme_code,academic_year,admission_date,seat_category,admission_status'
      ];
      for (let i = 1; i <= 65; i++) {
        lines.push(
          `APAAR-OVER-${String(i).padStart(4, '0')},ALPHA-OVER-${String(i).padStart(4, '0')},Student ${i},CSE-01,2026-27,2026-07-12,REGULAR,ACTIVE`
        );
      }
      csvData = lines.join('\n') + '\n';
    }

    const testFile = new File([csvData], fileName, { type: 'text/csv' });
    setFile(testFile);
  };

  async function onSubmit(event: React.FormEvent) {
    event.preventDefault();
    if (!file) {
      setError('Please select or load a CSV file first.');
      return;
    }
    setError(null);
    setBusy(true);
    try {
      const key = `${file.name}-${file.size}-${Date.now()}`;
      const batch = await api.uploadBatch(file, key);
      navigate(`/batches/${batch.id}`);
    } catch (err: any) {
      setError(err?.message || 'Batch upload and validation failed.');
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="space-y-6 max-w-4xl mx-auto">
      <div>
        <h1 className="text-2xl font-bold text-slate-900 flex items-center gap-2">
          <UploadCloud className="w-7 h-7 text-blue-600" />
          CSV Student Batch Ingestion
        </h1>
        <p className="text-sm text-slate-500 mt-1">
          Upload and validate institutional student admissions. Template schema is strictly versioned (<code>csv-v1</code>). No rows are discarded silently.
        </p>
      </div>

      {/* Template Specification Box */}
      <div className="panel bg-white border border-slate-200">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between pb-4 border-b border-slate-100 gap-3">
          <div>
            <h2 className="text-base font-semibold text-slate-800 flex items-center gap-2">
              <FileSpreadsheet className="w-5 h-5 text-emerald-600" />
              Standard CSV Template (<code>csv-v1</code>)
            </h2>
            <p className="text-xs text-slate-500 mt-0.5">
              Requires 8 mandatory columns. Limits: 5 MB file cap, 5,000 rows per batch.
            </p>
          </div>
          <button
            type="button"
            onClick={handleDownloadTemplate}
            className="btn-secondary text-xs self-start sm:self-auto"
          >
            <Download className="w-3.5 h-3.5" />
            Download Template
          </button>
        </div>

        <div className="mt-4 bg-slate-900 text-slate-100 p-3.5 rounded-lg text-xs font-mono overflow-x-auto">
          <code>apaar_id,institution_student_id,full_name,programme_code,academic_year,admission_date,seat_category,admission_status</code>
        </div>
      </div>

      {/* Upload Wizard Form */}
      <form onSubmit={onSubmit} className="panel bg-white border border-slate-200 space-y-6">
        <div>
          <h2 className="text-base font-semibold text-slate-800 mb-1">Upload Data File</h2>
          <p className="text-xs text-slate-500 mb-4">
            Select a local CSV file, or load one of the synthetic evaluator fixtures below.
          </p>

          {/* Quick Evaluator Fixture Buttons */}
          <div className="p-3 bg-blue-50/60 border border-blue-200 rounded-xl mb-4">
            <span className="text-xs font-bold text-blue-800 uppercase tracking-wider flex items-center gap-1.5 mb-2">
              <Sparkles className="w-3.5 h-3.5 text-blue-600" />
              Quick-Load Synthetic Fixtures (Evaluation)
            </span>
            <div className="flex flex-wrap gap-2 text-xs">
              <button
                type="button"
                onClick={() => loadSampleDataset('demo')}
                className="px-3 py-1.5 bg-white border border-blue-300 hover:bg-blue-50 rounded-lg text-blue-900 font-medium transition"
              >
                demo_batch.csv (Errors, Duplicates &amp; Warnings)
              </button>
              <button
                type="button"
                onClick={() => loadSampleDataset('clean')}
                className="px-3 py-1.5 bg-white border border-emerald-300 hover:bg-emerald-50 rounded-lg text-emerald-900 font-medium transition"
              >
                clean_batch.csv (Valid Records)
              </button>
              <button
                type="button"
                onClick={() => loadSampleDataset('overrun')}
                className="px-3 py-1.5 bg-white border border-amber-300 hover:bg-amber-50 rounded-lg text-amber-900 font-medium transition"
              >
                intake_overrun.csv (Capacity Exception 65/60)
              </button>
            </div>
          </div>

          {/* File Input */}
          <div className="border-2 border-dashed border-slate-300 hover:border-blue-500 rounded-xl p-6 text-center transition bg-slate-50/50">
            <input
              id="file-upload"
              type="file"
              accept=".csv,text/csv"
              onChange={(e) => setFile(e.target.files?.[0] ?? null)}
              className="hidden"
            />
            <label htmlFor="file-upload" className="cursor-pointer block">
              <UploadCloud className="w-10 h-10 text-slate-400 mx-auto mb-2" />
              <span className="text-sm font-semibold text-blue-600 hover:text-blue-700">
                Click to browse
              </span>{' '}
              <span className="text-sm text-slate-500">or drop CSV file here</span>
              <p className="text-xs text-slate-400 mt-1">UTF-8 encoded CSV up to 5 MB</p>
            </label>
          </div>

          {file && (
            <div className="mt-3 p-3 bg-slate-100 rounded-lg flex items-center justify-between text-xs font-mono text-slate-700">
              <span className="flex items-center gap-2">
                <FileText className="w-4 h-4 text-blue-600" />
                {file.name} ({Math.round(file.size / 1024)} KB)
              </span>
              <span className="text-emerald-700 font-semibold flex items-center gap-1 font-sans">
                <CheckCircle2 className="w-3.5 h-3.5" /> Ready for validation
              </span>
            </div>
          )}
        </div>

        {error && (
          <div className="p-3 bg-red-50 border border-red-200 rounded-lg text-xs text-red-700 flex items-center gap-2">
            <AlertCircle className="w-4 h-4 text-red-600 shrink-0" />
            <span>{error}</span>
          </div>
        )}

        <button
          type="submit"
          disabled={busy || !file}
          className="btn-primary w-full py-2.5 justify-center text-sm font-semibold shadow-md shadow-blue-500/20"
        >
          {busy ? 'Validating Batch...' : 'Upload and Validate Batch'}
        </button>
      </form>
    </div>
  );
}
