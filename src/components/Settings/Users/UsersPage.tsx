import React, { useCallback, useState } from 'react'
import type { UsersSnapshot, UserInfo } from '../../../types'
import { BackHeader } from '../../Layout'
import { UserList } from './UserList'
import { UserForm } from './UserForm'

type UsersPageProps = {
  users: UsersSnapshot | null
  onBack: () => void
  onCreate: (name: string) => Promise<void>
  onSwitch: (id: string) => Promise<void>
  onDelete: (id: string, deleteData: boolean) => Promise<void>
  onUpdate: (id: string, name: string) => Promise<void>
}

function UsersPageComponent({
  users,
  onBack,
  onCreate,
  onSwitch,
  onDelete,
  onUpdate,
}: UsersPageProps) {
  const [newUserName, setNewUserName] = useState('')
  const [deleteUserData, setDeleteUserData] = useState(false)
  const [editingUserId, setEditingUserId] = useState<string | null>(null)
  const [editingUserName, setEditingUserName] = useState('')

  const handleEdit = useCallback((user: UserInfo) => {
    setEditingUserId(user.id)
    setEditingUserName(user.name)
  }, [])

  const handleCancelEdit = useCallback(() => {
    setEditingUserId(null)
    setEditingUserName('')
  }, [])

  const handleSaveEdit = useCallback(async () => {
    if (!editingUserId || !editingUserName.trim()) return
    await onUpdate(editingUserId, editingUserName.trim())
    setEditingUserId(null)
    setEditingUserName('')
  }, [editingUserId, editingUserName, onUpdate])

  const handleCreate = useCallback(async () => {
    if (!newUserName.trim()) return
    await onCreate(newUserName.trim())
    setNewUserName('')
  }, [newUserName, onCreate])

  const handleDelete = useCallback(
    async (id: string) => {
      await onDelete(id, deleteUserData)
    },
    [deleteUserData, onDelete],
  )

  return (
    <div className="settings-page">
      <BackHeader title="User Profiles" subtitle="Settings" onBack={onBack} />
      <div className="settings-page-content">
        <div className="panel">
          <h3>User Management</h3>
          <p>Switch users to keep ratings and game files separated.</p>
          <UserList
            users={users?.users ?? []}
            activeUserId={users?.activeUser ?? null}
            editingUserId={editingUserId}
            editingUserName={editingUserName}
            onEdit={handleEdit}
            onSave={handleSaveEdit}
            onCancel={handleCancelEdit}
            onSwitch={onSwitch}
            onDelete={handleDelete}
            onEditNameChange={setEditingUserName}
          />
          <UserForm
            newUserName={newUserName}
            deleteUserData={deleteUserData}
            onNewUserNameChange={setNewUserName}
            onDeleteUserDataChange={setDeleteUserData}
            onCreate={handleCreate}
          />
        </div>
      </div>
    </div>
  )
}

export const UsersPage = React.memo(UsersPageComponent)
