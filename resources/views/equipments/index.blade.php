@extends('admin-layout.app')

@section('content')
    <div class="row">
        <div class="col-xl-12 box-col-6 px-3 py-2">
            <div class="card">
                <div class="card-body p-0">
                    <div class="card mx-3">
                        <div class="card-header pb-0 card-no-border d-flex justify-content-between align-items-center">
                            <div>
                                <h4>Equipments List</h4>
                                <span>See Equipments below.</span>
                            </div>
                            <!-- Add Button opens Add Modal -->
                            <button type="button" class="btn btn-primary btn-sm" data-bs-toggle="modal"
                                data-bs-target="#addEquipmentModal">
                                <i class="fa fa-plus"></i> Add Equipment
                            </button>
                        </div>

                        <div class="card-body">
                            <div class="table-responsive">
                                <div style="max-height:500px;">
                                    <table class="table table-striped w-100" id="user-approval-table">
                                        <thead style="position: sticky; top: 0; background: #fff; z-index: 2;">
                                            <tr>
                                                <th>S No.</th>
                                                <th>Name</th>
                                                <th>Action</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            @foreach ($equipments as $i => $equipment)
                                                <tr>
                                                    <td>{{ ++$i }}</td>
                                                    <td>{{ $equipment->name }}</td>
                                                    <td class="d-flex gap-1">
                                                        <!-- Edit Button -->
                                                        <button type="button" class="btn btn-primary btn-sm editBtn"
                                                            data-id="{{ $equipment->id }}" data-name="{{ $equipment->name }}"
                                                            data-action="{{ route('equipments.update', $equipment->id) }}">
                                                            <i class="fa fa-pencil"></i> Edit
                                                        </button>

                                                        <!-- Delete Button -->
                                                        <button type="button" class="btn btn-danger btn-sm deleteBtn"
                                                            data-id="{{ $equipment->id }}" data-name="{{ $equipment->name }}"
                                                            data-action="{{ route('equipments.destroy', $equipment->id) }}">
                                                            <i class="fa fa-trash"></i> Delete
                                                        </button>
                                                    </td>
                                                </tr>
                                            @endforeach
                                        </tbody>
                                    </table>
                                </div>
                            </div>
                        </div>

                    </div> <!-- /.card mx-3 -->
                </div>
            </div>
        </div>
    </div>

    <!-- ADD Equipment Modal -->
    <div class="modal fade" id="addEquipmentModal" tabindex="-1" aria-hidden="true">
        <div class="modal-dialog modal-dialog-centered" style="max-width: 500px;">
            <div class="modal-content p-4">
                <div class="modal-header border-0">
                    <h5 class="modal-title">Add Equipment</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">
                    <form method="POST" action="{{ route('equipments.store') }}">
                        @csrf
                        <div class="mb-3">
                            <label class="form-label">Name</label>
                            <input type="text" class="form-control" name="name" placeholder="Enter Equipment" required>
                        </div>
                        <div class="text-end">
                            <button type="submit" class="btn btn-primary btn-sm px-4">Save</button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    </div>

    <!-- EDIT Equipment Modal -->
    <div class="modal fade" id="editEquipmentModal" tabindex="-1" aria-hidden="true">
        <div class="modal-dialog modal-dialog-centered" style="max-width: 500px;">
            <div class="modal-content p-4">
                <div class="modal-header border-0">
                    <h5 class="modal-title">Edit Equipment</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">
                    <form id="editEquipmentForm" method="POST" action="#">
                        @csrf
                        @method('PUT')
                        <div class="mb-3">
                            <label class="form-label">Name</label>
                            <input type="text" class="form-control" id="edit-name" name="name" required>
                        </div>
                        <div class="text-end">
                            <button type="submit" class="btn btn-primary btn-sm px-4">Update</button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    </div>

    <!-- DELETE Confirmation Modal -->
    <div class="modal fade" id="deleteEquipmentModal" tabindex="-1" aria-hidden="true">
        <div class="modal-dialog modal-dialog-centered" style="max-width: 450px;">
            <div class="modal-content p-4">
                <div class="modal-header border-0">
                    <h5 class="modal-title text-danger">Confirm Delete</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">
                    <p>Are you sure you want to delete <strong id="delete-item-name"></strong>?</p>
                    <form id="deleteEquipmentForm" method="POST" action="#">
                        @csrf
                        @method('DELETE')
                        <div class="text-end">
                            <button type="button" class="btn btn-secondary btn-sm" data-bs-dismiss="modal">Cancel</button>
                            <button type="submit" class="btn btn-danger btn-sm px-4">Delete</button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    </div>

    {{-- Inline script --}}
    <script>
        document.addEventListener('click', function (e) {
            // EDIT
            if (e.target.closest('.editBtn')) {
                const btn = e.target.closest('.editBtn');
                const name = btn.getAttribute('data-name');
                const action = btn.getAttribute('data-action');

                document.getElementById('edit-name').value = name || '';
                document.getElementById('editEquipmentForm').setAttribute('action', action);

                bootstrap.Modal.getOrCreateInstance(document.getElementById('editEquipmentModal')).show();
            }

            // DELETE
            if (e.target.closest('.deleteBtn')) {
                const btn = e.target.closest('.deleteBtn');
                const name = btn.getAttribute('data-name');
                const action = btn.getAttribute('data-action');

                document.getElementById('delete-item-name').textContent = name || '';
                document.getElementById('deleteEquipmentForm').setAttribute('action', action);

                bootstrap.Modal.getOrCreateInstance(document.getElementById('deleteEquipmentModal')).show();
            }
        });
    </script>
@endsection