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
                            <a href="{{ route('equipments.create') }}" class="btn btn-primary btn-sm">
                                <i class="fa fa-plus"></i> Add Equipments
                            </a>
                        </div>
                        <div class="card-body">
                            <div class="table-responsive">
                                <div style="max-height:500px; overflow-y:auto;">
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
                                                    <td>
                                                        <a class="btn btn-primary btn-sm"
                                                            href="{{ route('equipments.edit', $equipment->id) }}">
                                                            <i class="fa fa-pencil"></i> Edit
                                                        </a>
                                                        <form method="POST"
                                                            action="{{ route('equipments.destroy', $equipment->id) }}"
                                                            style="display:inline">
                                                            @csrf
                                                            @method('DELETE')
                                                            <button type="submit" class="btn btn-danger btn-sm">
                                                                <i class="fa-solid fa-trash"></i> Delete
                                                            </button>
                                                        </form>
                                                    </td>
                                                </tr>
                                            @endforeach
                                        </tbody>
                                    </table>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
@endsection
