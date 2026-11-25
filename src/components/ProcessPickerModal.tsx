import React, { useMemo, useState } from "react";
import { RefreshCw, Search, X, Plus, Check } from "lucide-react";
import { Language, RunningProcessInfo } from "../types";
import { t } from "../i18n";

interface ProcessPickerModalProps {
  mode: "game" | "background";
  lang: Language;
  processes: RunningProcessInfo[];
  loading: boolean;
  onRefresh: () => void;
  onSelect: (exeName: string) => void;
  onClose: () => void;
}

export const ProcessPickerModal: React.FC<ProcessPickerModalProps> = ({
  mode,
  lang,
  processes,
  loading,
  onRefresh,
  onSelect,
  onClose,
}) => {
  const dict = t(lang);
  const [query, setQuery] = useState("");

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return processes;
    return processes.filter((p) => p.exeName.toLowerCase().includes(q));
  }, [processes, query]);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm p-4">
      <div className="w-full max-w-xl rounded-2xl border border-slate-800 bg-[#0d1322] shadow-2xl overflow-hidden flex flex-col max-h-[82vh]">
        <div className="flex items-center justify-between px-5 py-4 border-b border-slate-800/80">
          <h3 className="text-base font-semibold text-slate-100">
            {mode === "game" ? dict.pickerTitleGame : dict.pickerTitleBg}
          </h3>
          <button
            onClick={onClose}
            className="p-1.5 rounded-lg text-slate-400 hover:text-slate-100 hover:bg-slate-800 transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="px-5 py-3 border-b border-slate-800/60 flex items-center gap-2.5">
          <div className="relative flex-1">
            <Search className="w-4 h-4 text-slate-400 absolute left-3 top-1/2 -translate-y-1/2" />
            <input
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder={dict.searchPlaceholder}
              className="w-full pl-9 pr-3 py-2 text-sm rounded-xl bg-slate-900/90 border border-slate-700/70 text-slate-100 placeholder:text-slate-500 focus:outline-none focus:border-cyan-500/70"
            />
          </div>
          <button
            onClick={onRefresh}
            disabled={loading}
            className="inline-flex items-center gap-1.5 px-3 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-medium transition-colors disabled:opacity-50"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${loading ? "animate-spin" : ""}`} />
            {dict.refresh}
          </button>
        </div>

        <div className="flex-1 overflow-y-auto divide-y divide-slate-800/60 px-2 py-1">
          {filtered.map((proc) => {
            const alreadyAdded =
              mode === "game" ? proc.isTrackedGame : proc.isBackgroundApp;
            return (
              <div
                key={proc.exeName}
                className="flex items-center justify-between px-3 py-2.5 rounded-xl hover:bg-slate-800/50 transition-colors"
              >
                <div className="min-w-0">
                  <div className="text-sm font-medium text-slate-100 truncate">
                    {proc.exeName}
                  </div>
                  <div className="text-xs text-slate-400 flex items-center gap-2 mt-0.5">
                    <span>{proc.memoryMb} MB RAM</span>
                    <span>•</span>
                    <span>
                      {proc.instanceCount} {dict.instances}
                    </span>
                  </div>
                </div>

                {alreadyAdded ? (
                  <span className="inline-flex items-center gap-1 px-2.5 py-1 rounded-lg bg-emerald-500/15 text-emerald-300 text-xs font-medium border border-emerald-500/25">
                    <Check className="w-3.5 h-3.5" />
                    {dict.added}
                  </span>
                ) : (
                  <button
                    onClick={() => onSelect(proc.exeName)}
                    className="inline-flex items-center gap-1 px-3 py-1.5 rounded-lg bg-cyan-500/15 hover:bg-cyan-500/25 text-cyan-300 border border-cyan-500/30 text-xs font-medium transition-colors"
                  >
                    <Plus className="w-3.5 h-3.5" />
                    {dict.add}
                  </button>
                )}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};
