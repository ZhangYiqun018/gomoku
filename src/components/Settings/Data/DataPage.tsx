import React from 'react'
import { BackHeader } from '../../Layout'
import { ExportPanel } from './ExportPanel'

type DataPageProps = {
  activeUserDir: string | null
  onBack: () => void
  onExport: () => void
}

function DataPageComponent({ activeUserDir, onBack, onExport }: DataPageProps) {
  return (
    <div className="settings-page">
      <BackHeader title="Data Management" subtitle="Settings" onBack={onBack} />
      <div className="settings-page-content">
        <ExportPanel activeUserDir={activeUserDir} onExport={onExport} />
      </div>
    </div>
  )
}

export const DataPage = React.memo(DataPageComponent)
