import React from 'react'

type Column<T> = {
  key: string
  header: string
  render?: (item: T) => React.ReactNode
  accessor?: keyof T
}

type TableProps<T> = {
  columns: Column<T>[]
  data: T[]
  getRowKey: (item: T) => string
  activeKey?: string | null
  className?: string
}

function TableComponent<T>({ columns, data, getRowKey, activeKey, className = '' }: TableProps<T>) {
  return (
    <div className="table-wrap">
      <table className={`profile-table ${className}`.trim()}>
        <thead>
          <tr>
            {columns.map((col) => (
              <th key={col.key}>{col.header}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {data.map((item) => {
            const key = getRowKey(item)
            return (
              <tr key={key} className={key === activeKey ? 'active' : ''}>
                {columns.map((col) => (
                  <td key={col.key}>
                    {col.render
                      ? col.render(item)
                      : col.accessor
                      ? String(item[col.accessor] ?? '—')
                      : '—'}
                  </td>
                ))}
              </tr>
            )
          })}
        </tbody>
      </table>
    </div>
  )
}

export const Table = React.memo(TableComponent) as <T>(props: TableProps<T>) => React.ReactElement
