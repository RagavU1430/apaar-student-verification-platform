import { useEffect, useState } from 'react';
import { api, type Admission } from '../api/client';
import StatusBadge from '../components/StatusBadge';
import { Search, Download, UserCheck, ShieldCheck } from 'lucide-react';

export default function AdmissionsPage() {
  const [admissions, setAdmissions] = useState<Admission[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState('');
  const [programmeFilter, setProgrammeFilter] = useState('');
  const [categoryFilter, setCategoryFilter] = useState('');
  const [statusFilter, setStatusFilter] = useState('');

  const fetchRecords = () => {
    setLoading(true);
    api
      .admissions({
        search: search || undefined,
        programmeCode: programmeFilter || undefined,
        seatCategory: categoryFilter || undefined,
        status: statusFilter || undefined
      })
      .then((res) => setAdmissions(res.items))
      .catch((err) => console.error(err))
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    fetchRecords();
  }, [programmeFilter, categoryFilter, statusFilter]);

  const handleSearchSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    fetchRecords();
  };

  const handleExportCsv = () => {
    if (admissions.length === 0) return;
    const headers = ['Student ID', 'APAAR ID (Masked)', 'Full Name', 'Programme', 'Academic Year', 'Admission Date', 'Seat Category', 'Status'];
    const rows = admissions.map((a) => [
      a.institution_student_id,
      a.apaar_id_masked,
      `"${a.full_name || ''}"`,
      a.programme_code,
      a.academic_year,
      a.admission_date,
      a.seat_category,
      a.status
    ]);
    const csvContent = [headers.join(','), ...rows.map((r) => r.join(','))].join('\n');
    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.setAttribute('download', `admissions_export_${Date.now()}.csv`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };

  return (
    <div className="space-y-6">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold text-slate-900 flex items-center gap-2">
            <UserCheck className="w-7 h-7 text-blue-600" />
            Student &amp; Admission Registry
          </h1>
          <p className="text-sm text-slate-500 mt-1">
            Authorised student records linked to APAAR IDs. APAAR numbers are masked in accordance with privacy rules.
          </p>
        </div>
        <button
          onClick={handleExportCsv}
          disabled={admissions.length === 0}
          className="btn-secondary self-start md:self-auto"
        >
          <Download className="w-4 h-4 text-slate-600" />
          Export CSV ({admissions.length})
        </button>
      </div>

      {/* Filter Toolbar */}
      <div className="panel p-4 bg-white border border-slate-200">
        <form onSubmit={handleSearchSubmit} className="grid grid-cols-1 md:grid-cols-5 gap-3">
          <div className="md:col-span-2 relative">
            <Search className="w-4 h-4 absolute left-3 top-3 text-slate-400" />
            <input
              type="text"
              placeholder="Search by student ID, name, or APAAR..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="w-full pl-9 pr-3 py-2 text-sm border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          <div>
            <select
              value={programmeFilter}
              onChange={(e) => setProgrammeFilter(e.target.value)}
              className="w-full px-3 py-2 text-sm border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white"
            >
              <option value="">All Programmes</option>
              <option value="CSE-01">CSE-01 (Computer Science)</option>
              <option value="ECE-01">ECE-01 (Electronics)</option>
              <option value="MECH-01">MECH-01 (Mechanical)</option>
            </select>
          </div>
          <div>
            <select
              value={categoryFilter}
              onChange={(e) => setCategoryFilter(e.target.value)}
              className="w-full px-3 py-2 text-sm border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white"
            >
              <option value="">All Categories</option>
              <option value="REGULAR">REGULAR</option>
              <option value="SUPERNUMERARY">SUPERNUMERARY</option>
              <option value="NRI">NRI</option>
              <option value="OCI">OCI</option>
              <option value="FOREIGN_NATIONAL">FOREIGN NATIONAL</option>
            </select>
          </div>
          <div className="flex gap-2">
            <button type="submit" className="btn-primary flex-1 justify-center">
              Filter
            </button>
            {(search || programmeFilter || categoryFilter || statusFilter) && (
              <button
                type="button"
                onClick={() => {
                  setSearch('');
                  setProgrammeFilter('');
                  setCategoryFilter('');
                  setStatusFilter('');
                }}
                className="btn-secondary"
              >
                Reset
              </button>
            )}
          </div>
        </form>
      </div>

      {/* Table */}
      <div className="panel overflow-hidden p-0">
        {loading ? (
          <div className="p-8 text-center text-slate-500">Loading admissions registry...</div>
        ) : admissions.length === 0 ? (
          <div className="p-12 text-center">
            <ShieldCheck className="w-12 h-12 text-slate-300 mx-auto mb-3" />
            <h3 className="font-semibold text-slate-700">No student admissions found</h3>
            <p className="text-sm text-slate-500 mt-1 max-w-md mx-auto">
              No admissions match the chosen filter or no batches have been committed yet. Upload and commit a batch from the upload tab.
            </p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left border-collapse">
              <thead>
                <tr className="bg-slate-50 border-b border-slate-200 text-xs font-semibold text-slate-600 uppercase tracking-wider">
                  <th className="py-3 px-4">Student ID</th>
                  <th className="py-3 px-4">APAAR ID</th>
                  <th className="py-3 px-4">Full Name</th>
                  <th className="py-3 px-4">Programme</th>
                  <th className="py-3 px-4">Year</th>
                  <th className="py-3 px-4">Date</th>
                  <th className="py-3 px-4">Seat Category</th>
                  <th className="py-3 px-4">Status</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100 text-sm">
                {admissions.map((record) => (
                  <tr key={record.id} className="hover:bg-slate-50/80 transition-colors">
                    <td className="py-3 px-4 font-mono font-medium text-slate-900">
                      {record.institution_student_id}
                    </td>
                    <td className="py-3 px-4 font-mono text-slate-600 text-xs">
                      {record.apaar_id_masked}
                    </td>
                    <td className="py-3 px-4 font-medium text-slate-800">
                      {record.full_name || <span className="text-slate-400 italic">Not provided</span>}
                    </td>
                    <td className="py-3 px-4 font-mono text-xs text-blue-600 bg-blue-50/50 rounded inline-block my-2">
                      {record.programme_code}
                    </td>
                    <td className="py-3 px-4 text-slate-600">{record.academic_year}</td>
                    <td className="py-3 px-4 text-slate-600">{record.admission_date}</td>
                    <td className="py-3 px-4">
                      <span className="text-xs font-semibold px-2 py-0.5 rounded bg-slate-100 text-slate-700">
                        {record.seat_category}
                      </span>
                    </td>
                    <td className="py-3 px-4">
                      <StatusBadge label={record.status} />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
