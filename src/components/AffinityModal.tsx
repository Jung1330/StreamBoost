import React, { useState } from "react";
import { Cpu, X } from "lucide-react";
import { GameProfile, Language } from "../types";
import { t } from "../i18n";

interface AffinityModalProps {
  profile: GameProfile;
  logicalCores: number;
  lang: Language;
  onSave: (cores: number[] | null) => void;
  onClose: () => void;
}

export const AffinityModal: React.FC<AffinityModalProps> = ({
  profile,
  logicalCores,
  lang,
  onSave,
  onClose,
}) => {
  const dict = t(lang);
  const allCoreIndices = Array.from({ length: logicalCores }, (_, i) => i);

  const [useAll, setUseAll] = useState<boolean>(profile.affinityCores === null);
  const [selected, setSelected] = useState<number[]>(
    profile.affinityCores ?? allCoreIndices
  );

  const toggleCore = (idx: number) => {
    setUseAll(false);
    setSelected((prev) => {
      if (prev.includes(idx)) {
        if (prev.length <= 1) return prev; // keep at least 1 core selected
        return prev.filter((c) => c !== idx);
      }
      return [...prev, idx].sort((a, b) => a - b);
    });
  };

  const applyPreset = (cores: number[]) => {
    setUseAll(false);
    setSelected(cores.length > 0 ? cores : [0]);
  };

  const handleSave = () => {
    if (useAll || selected.length === logicalCores) {
      onSave(null);
    } else {
      onSave(selected);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm p-4">
      <div className="w-full max-w-lg rounded-2xl border border-slate-800 bg-[#0d1322] shadow-2xl overflow-hidden">
        <div className="flex items-center justify-between px-5 py-4 border-b border-slate-800/80">
          <div className="flex items-center gap-2.5">
            <div className="p-2 rounded-xl bg-cyan-500/15 text-cyan-400">
              <Cpu className="w-4 h-4" />
            </div>
            <div>
              <h3 className="text-sm font-semibold text-slate-100">
                {dict.affinityModalTitle}
              </h3>
              <p className="text-xs text-slate-400">{profile.name}</p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 rounded-lg text-slate-400 hover:text-slate-100 hover:bg-slate-800 transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="p-5 space-y-4">
          <p className="text-xs text-slate-400 leading-relaxed">
            {dict.affinityModalDesc}
          </p>

          <label className="flex items-center gap-3 p-3 rounded-xl bg-slate-900/80 border border-slate-800 cursor-pointer">
            <input
              type="checkbox"
              checked={useAll}
              onChange={(e) => {
                setUseAll(e.target.checked);
                if (e.target.checked) setSelected(allCoreIndices);
              }}
              className="w-4 h-4 accent-cyan-500 rounded"
            />
            <span className="text-sm font-medium text-slate-200">
              {dict.useAllCores}
            </span>
          </label>

          <div className="flex flex-wrap gap-2">
            <button
              type="button"
              onClick={() =>
                applyPreset(allCoreIndices.filter((c) => c % 2 === 0))
              }
              className="px-2.5 py-1.5 rounded-lg text-xs bg-slate-800/80 hover:bg-slate-700 text-slate-300 border border-slate-700/70 transition-colors"
            >
              {dict.selectEvenCores}
            </button>
            <button
              type="button"
              onClick={() =>
                applyPreset(allCoreIndices.slice(0, Math.ceil(logicalCores / 2)))
              }
              className="px-2.5 py-1.5 rounded-lg text-xs bg-slate-800/80 hover:bg-slate-700 text-slate-300 border border-slate-700/70 transition-colors"
            >
              {dict.selectFirstHalf}
            </button>
            <button
              type="button"
              onClick={() =>
                applyPreset(allCoreIndices.slice(Math.floor(logicalCores / 2)))
              }
              className="px-2.5 py-1.5 rounded-lg text-xs bg-slate-800/80 hover:bg-slate-700 text-slate-300 border border-slate-700/70 transition-colors"
            >
              {dict.selectSecondHalf}
            </button>
          </div>

          <div className="grid grid-cols-4 sm:grid-cols-6 gap-2 max-h-48 overflow-y-auto pr-1">
            {allCoreIndices.map((idx) => {
              const active = useAll || selected.includes(idx);
              return (
                <button
                  key={idx}
                  type="button"
                  onClick={() => toggleCore(idx)}
                  className={`py-2 rounded-xl text-xs font-semibold border transition-all ${
                    active
                      ? "bg-cyan-500/20 border-cyan-400/60 text-cyan-200 shadow-sm shadow-cyan-500/10"
                      : "bg-slate-900/70 border-slate-800 text-slate-500 hover:border-slate-700"
                  }`}
                >
                  CPU {idx}
                </button>
              );
            })}
          </div>
        </div>

        <div className="px-5 py-3.5 border-t border-slate-800/80 flex items-center justify-end gap-2.5 bg-slate-900/40">
          <button
            onClick={onClose}
            className="px-4 py-2 rounded-xl text-xs font-medium text-slate-300 hover:bg-slate-800 transition-colors"
          >
            {dict.cancel}
          </button>
          <button
            onClick={handleSave}
            className="px-4 py-2 rounded-xl text-xs font-semibold bg-cyan-500 hover:bg-cyan-400 text-slate-950 transition-colors"
          >
            {dict.save}
          </button>
        </div>
      </div>
    </div>
  );
};
