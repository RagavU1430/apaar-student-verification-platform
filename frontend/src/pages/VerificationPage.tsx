import { useState } from 'react';
import { api, type VerificationResponse } from '../api/client';
import StatusBadge from '../components/StatusBadge';
import { ShieldCheck, AlertCircle, Info, Search, FileBadge } from 'lucide-react';
import { PURPOSE_POLICIES } from '../lib/verification';

export default function VerificationPage() {
  const [purpose, setPurpose] = useState<string>('ENROLMENT_STATUS');
  const [recordRef, setRecordRef] = useState('');
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<VerificationResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleVerify = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!recordRef.trim()) return;

    setLoading(true);
    setError(null);
    setResult(null);

    try {
      const response = await api.verify({
        purpose: purpose as any,
        record_ref: recordRef.trim()
      });
      setResult(response);
    } catch (err: any) {
      setError(err.message || 'Verification request failed.');
    } finally {
      setLoading(false);
    }
  };

  const currentPolicy = PURPOSE_POLICIES[purpose];

  return (
    <div className="space-y-6 max-w-4xl mx-auto">
      <div>
        <h1 className="text-2xl font-bold text-slate-900 flex items-center gap-2">
          <ShieldCheck className="w-7 h-7 text-emerald-600" />
          Authorised Verification Service
        </h1>
        <p className="text-sm text-slate-500 mt-1">
          Perform purpose-limited academic identity verification. Access is logged and only approved fields are disclosed.
        </p>
      </div>

      {/* Verification Query Form */}
      <div className="panel bg-white border border-slate-200">
        <h2 className="text-lg font-semibold text-slate-800 mb-4 flex items-center gap-2">
          <Search className="w-5 h-5 text-blue-600" />
          New Verification Request
        </h2>

        <form onSubmit={handleVerify} className="space-y-4">
          <div>
            <label className="block text-xs font-semibold uppercase text-slate-600 mb-1.5">
              Verification Purpose (Required Policy Scope)
            </label>
            <select
              value={purpose}
              onChange={(e) => setPurpose(e.target.value)}
              className="w-full px-3.5 py-2.5 text-sm border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 bg-white"
            >
              {Object.values(PURPOSE_POLICIES).map((p) => (
                <option key={p.purpose} value={p.purpose}>
                  {p.label}
                </option>
              ))}
            </select>
            {currentPolicy && (
              <p className="text-xs text-slate-500 mt-1.5">
                {currentPolicy.description} <strong className="text-slate-700">Disclosed fields:</strong> {currentPolicy.allowedFields.join(', ')}.
              </p>
            )}
          </div>

          <div>
            <label className="block text-xs font-semibold uppercase text-slate-600 mb-1.5">
              Record Reference (Institution Student ID or Masked APAAR ID)
            </label>
            <div className="flex gap-2">
              <input
                type="text"
                placeholder="e.g. ALPHA-0001 or APAAR-DEMO-0001"
                value={recordRef}
                onChange={(e) => setRecordRef(e.target.value)}
                className="flex-1 px-3.5 py-2.5 text-sm border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 font-mono"
                required
              />
              <button
                type="submit"
                disabled={loading || !recordRef.trim()}
                className="btn-primary px-6"
              >
                {loading ? 'Verifying...' : 'Verify Record'}
              </button>
            </div>
          </div>
        </form>

        {error && (
          <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg text-sm text-red-700 flex items-center gap-2">
            <AlertCircle className="w-4 h-4 shrink-0 text-red-600" />
            <span>{error}</span>
          </div>
        )}
      </div>

      {/* Verification Result Card */}
      {result && (
        <div className="panel border-2 border-slate-300 bg-white p-6 shadow-sm">
          <div className="flex flex-col sm:flex-row sm:items-center justify-between pb-4 border-b border-slate-200 gap-3">
            <div>
              <span className="text-xs uppercase font-bold tracking-wider text-slate-500">
                Verification Outcome
              </span>
              <div className="flex items-center gap-3 mt-1">
                <StatusBadge label={result.outcome} className="text-sm px-3 py-1" />
                <span className="text-xs font-mono text-slate-500">
                  Ref: {recordRef}
                </span>
              </div>
            </div>

            {/* Prominent Simulated Adapter Notice */}
            <div className="bg-amber-50 border border-amber-300 text-amber-800 text-xs px-3 py-1.5 rounded-md flex items-center gap-2 self-start sm:self-auto font-medium">
              <Info className="w-4 h-4 text-amber-600 shrink-0" />
              <span>Simulated Prototype Adapter (A-02)</span>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 my-6 text-sm">
            <div>
              <span className="text-xs text-slate-500 uppercase font-semibold">Data Source</span>
              <p className="font-mono text-slate-800 mt-0.5">{result.source}</p>
            </div>
            <div>
              <span className="text-xs text-slate-500 uppercase font-semibold">Verification Timestamp</span>
              <p className="text-slate-800 mt-0.5">{new Date(result.verified_at).toLocaleString()}</p>
            </div>
          </div>

          {/* Disclosed Fields Table */}
          <div className="mt-4">
            <h3 className="text-sm font-semibold text-slate-800 mb-2 flex items-center gap-2">
              <FileBadge className="w-4 h-4 text-blue-600" />
              Permitted Disclosed Fields ({result.fields_disclosed.length})
            </h3>

            {result.fields_disclosed.length === 0 ? (
              <p className="text-sm text-slate-500 italic p-3 bg-slate-50 rounded">
                No matching active student admission found under this reference.
              </p>
            ) : (
              <div className="border border-slate-200 rounded-lg overflow-hidden">
                <table className="w-full text-left m-0">
                  <thead>
                    <tr className="bg-slate-50 text-xs text-slate-600 uppercase">
                      <th className="py-2.5 px-4">Field</th>
                      <th className="py-2.5 px-4">Value</th>
                      <th className="py-2.5 px-4">Policy Basis</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-100 text-sm">
                    {result.fields_disclosed.map((f) => (
                      <tr key={f.name}>
                        <td className="py-2.5 px-4 font-mono text-xs text-slate-600">{f.name}</td>
                        <td className="py-2.5 px-4 font-semibold text-slate-900">{f.value}</td>
                        <td className="py-2.5 px-4 text-xs text-slate-500">{f.justification}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>

          {/* Limitations and Warnings */}
          <div className="mt-6 p-4 bg-slate-50 rounded-lg border border-slate-200 text-xs text-slate-600 space-y-1.5">
            <h4 className="font-semibold text-slate-700 uppercase tracking-wide text-[11px]">
              Assurance Limitations &amp; Governance Notice
            </h4>
            <ul className="list-disc pl-4 space-y-1">
              {result.limitations.map((limit, idx) => (
                <li key={idx}>{limit}</li>
              ))}
            </ul>
          </div>
        </div>
      )}
    </div>
  );
}
