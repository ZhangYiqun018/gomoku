import React from 'react'
import type { UserInfo } from '../../../types'
import { Button } from '../../Shared'

type UserListProps = {
  users: UserInfo[]
  activeUserId: string | null
  editingUserId: string | null
  editingUserName: string
  onEdit: (user: UserInfo) => void
  onSave: () => void
  onCancel: () => void
  onSwitch: (id: string) => void
  onDelete: (id: string) => void
  onEditNameChange: (value: string) => void
}

function UserListComponent({
  users,
  activeUserId,
  editingUserId,
  editingUserName,
  onEdit,
  onSave,
  onCancel,
  onSwitch,
  onDelete,
  onEditNameChange,
}: UserListProps) {
  return (
    <div className="user-grid">
      {users.map((user) => (
        <div key={user.id} className={`user-card ${user.id === activeUserId ? 'active' : ''}`}>
          <div className="user-info">
            {editingUserId === user.id ? (
              <input
                type="text"
                value={editingUserName}
                onChange={(e) => onEditNameChange(e.target.value)}
              />
            ) : (
              <strong>{user.name}</strong>
            )}
            <span className="user-meta">ID: {user.id}</span>
            <span className="user-meta">Data: {user.dataDir}</span>
          </div>
          <div className="user-actions">
            {editingUserId === user.id ? (
              <>
                <Button variant="primary" onClick={onSave}>
                  Save
                </Button>
                <Button onClick={onCancel}>Cancel</Button>
              </>
            ) : (
              <>
                <Button onClick={() => onEdit(user)}>Edit</Button>
                <Button onClick={() => onSwitch(user.id)} disabled={user.id === activeUserId}>
                  {user.id === activeUserId ? 'Active' : 'Use'}
                </Button>
                <Button variant="danger" onClick={() => onDelete(user.id)}>
                  Delete
                </Button>
              </>
            )}
          </div>
        </div>
      ))}
    </div>
  )
}

export const UserList = React.memo(UserListComponent)
