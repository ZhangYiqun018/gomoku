import React from 'react'
import { Button } from '../../Shared'

type ExportPanelProps = {
  activeUserDir: string | null
  onExport: () => void
}

function ExportPanelComponent({ activeUserDir, onExport }: ExportPanelProps) {
  return (
    <div className="panel">
      <h3>Training Export</h3>
      <p>Export move-by-move training samples for future learning modules.</p>
      {activeUserDir && <p className="muted">Default folder: {activeUserDir}</p>}
      <div className="action-grid">
        <Button variant="primary" onClick={onExport}>
          Export training data
        </Button>
      </div>
    </div>
  )
}

export const ExportPanel = React.memo(ExportPanelComponent)
