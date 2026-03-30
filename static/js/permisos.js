const PermisosManager = {
    allModulos: [],
    currentPermisos: [],

    async init() {
        // Carga inicial de perfiles y la lista base de módulos
        await this.loadCatalogos();
    },

    async loadCatalogos() {
        try {
            const [resP, resM] = await Promise.all([
                fetch('/api/perfiles'), // ✅ CORREGIDO: Estaba como /api/perfil
                fetch('/api/modulos')   // ⚠️ ADVERTENCIA: Aún no hemos creado esta ruta en Rust
            ]);

            const perfiles = await resP.json();
            this.allModulos = await resM.json();

            // 1. Llenar el select de perfiles
            const select = document.getElementById('selectPerfil');
            select.innerHTML = '<option value="">Seleccione un perfil...</option>' +
                perfiles.map(p => {
                    const id = p.id ?? p.ID ?? p.idPerfil;
                    const nombre = p.strNombrePerfil ?? p.Nombre ?? p.strNombre;
                    return `<option value="${id}">${nombre}</option>`;
                }).join('');

            // 2. Inicializar matriz vacía (todos en false) para MOSTRAR LOS MÓDULOS INMEDIATAMENTE
            this.resetPermisos();
            
            // Renderizar la tabla (pasamos "false" para que los checks nazcan deshabilitados hasta elegir un perfil)
            this.renderTable(false);

        } catch (e) {
            console.error("Error al cargar catálogos:", e);
            this.showToast("Error al conectar con el servidor", 'error');
        }
    },

    resetPermisos() {
        // Genera la estructura base de permisos usando la lista de módulos
        this.currentPermisos = this.allModulos.map(m => ({
            idModulo: m.id,
            strNombreModulo: m.strNombreModulo,
            bitAgregar: false, bitEditar: false, bitEliminar: false,
            bitConsulta: false, bitDetalle: false, bitImprimir: false, bitBitacora: false
        }));
    },

    async loadPermisosPerfil() {
        const idPerfil = document.getElementById('selectPerfil').value;

        // Si regresan al "Seleccione un perfil", reseteamos la tabla y deshabilitamos
        if (!idPerfil) {
            this.resetPermisos();
            this.renderTable(false);
            return;
        }

        const tbody = document.getElementById('permisosTableBody');
        tbody.innerHTML = '<tr><td colspan="9" class="text-center"><i class="fas fa-spinner fa-spin"></i> Cargando permisos del perfil...</td></tr>';

        try {
            const res = await fetch(`/api/permisos_perfil/${idPerfil}`);
            const permisosDB = await res.json();

            // Sincronizar los permisos traídos de la BD con nuestra lista completa de módulos
            this.currentPermisos = this.allModulos.map(modulo => {
                const p = permisosDB.find(x => x.idModulo === modulo.id);
                return {
                    idModulo: modulo.id,
                    strNombreModulo: modulo.strNombreModulo,
                    bitAgregar: p ? p.bitAgregar : false,
                    bitEditar: p ? p.bitEditar : false,
                    bitEliminar: p ? p.bitEliminar : false,
                    bitConsulta: p ? p.bitConsulta : false,
                    bitDetalle: p ? p.bitDetalle : false,
                    bitImprimir: p ? p.bitImprimir : false,
                    bitBitacora: p ? p.bitBitacora : false
                };
            });

            // Renderizar tabla (pasamos "true" para habilitar los checkboxes)
            this.renderTable(true);
        } catch (e) {
            this.showToast("Error al cargar permisos específicos", 'error');
        }
    },

    renderTable(isProfileSelected) {
        const tbody = document.getElementById('permisosTableBody');

        tbody.innerHTML = this.currentPermisos.map(p => {
            
            // --- LÓGICA DE PERMISOS PARA BOTÓN ELIMINAR/LIMPIAR ---
            let btnEliminar = '';
            if (typeof PERMISOS_MODULO !== 'undefined' && PERMISOS_MODULO.canDelete) {
                btnEliminar = `
                    <button class="btn-icon" title="Limpiar" onclick="PermisosManager.deleteRow(${p.idModulo})" style="color: #dc3545; border: none; background: none; cursor: pointer; font-size: 1.1em;">
                        <i class="fas fa-trash-alt"></i>
                    </button>
                `;
            } else {
                btnEliminar = '<span class="text-muted small">Sin permisos</span>';
            }

            return `
            <tr>
                <td class="bold">${p.strNombreModulo}</td>
                <td class="text-center">${this.renderCheck(p.idModulo, 'bitAgregar', p.bitAgregar, isProfileSelected)}</td>
                <td class="text-center">${this.renderCheck(p.idModulo, 'bitEditar', p.bitEditar, isProfileSelected)}</td>
                <td class="text-center">${this.renderCheck(p.idModulo, 'bitEliminar', p.bitEliminar, isProfileSelected)}</td>
                <td class="text-center">${this.renderCheck(p.idModulo, 'bitConsulta', p.bitConsulta, isProfileSelected)}</td>
                <td class="text-center">${this.renderCheck(p.idModulo, 'bitDetalle', p.bitDetalle, isProfileSelected)}</td>
                <td class="text-center">${this.renderCheck(p.idModulo, 'bitImprimir', p.bitImprimir, isProfileSelected)}</td>
                <td class="text-center">${this.renderCheck(p.idModulo, 'bitBitacora', p.bitBitacora, isProfileSelected)}</td>
                <td class="text-center">
                    <div class="action-buttons" style="display: flex; gap: 8px; justify-content: center;">
                        ${btnEliminar}
                    </div>
                </td>
            </tr>
            `;
        }).join('');
    },

    renderCheck(idModulo, field, value, isProfileSelected) {
        // --- LÓGICA DE PERMISOS PARA CHECKBOXES ---
        // Se habilitan SOLO si seleccionaron un perfil Y además tienen permiso de edición
        const canEdit = typeof PERMISOS_MODULO !== 'undefined' ? PERMISOS_MODULO.canEdit : true;
        const isEnabled = isProfileSelected && canEdit;

        return `
            <input type="checkbox" 
                style="transform: scale(1.3); cursor: ${isEnabled ? 'pointer' : 'not-allowed'};"
                ${value ? 'checked' : ''} 
                ${!isEnabled ? 'disabled title="Sin perfil seleccionado o sin permisos de edición"' : ''}
                onchange="PermisosManager.autoSave(${idModulo}, '${field}', this.checked)">
        `;
    },

    // Función de autoguardado que se dispara con cada clic
    async autoSave(idModulo, field, isChecked) {
        const idPerfil = document.getElementById('selectPerfil').value;
        if (!idPerfil) return;

        // Actualizar el valor en nuestra memoria local
        const item = this.currentPermisos.find(p => p.idModulo === idModulo);
        if (item) item[field] = isChecked;

        try {
            // El backend de rust espera un array 'permisos', enviamos solo el renglón modificado
            const res = await fetch('/api/permisos_perfil', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    id_perfil: parseInt(idPerfil), // ✅ CORREGIDO: Ajustado al DTO de Rust
                    permisos: [item] 
                })
            });

            const result = await res.json();
            if (result.success) {
                // Toaster de éxito estilo Departamentos
                this.showToast('Permiso guardado correctamente', 'success');
            } else {
                throw new Error(result.msg || "Error desconocido");
            }
        } catch (e) {
            // Revertir el cambio local visualmente si la API falla
            item[field] = !isChecked;
            this.renderTable(true);
            this.showToast("Error al guardar: " + e.message, 'error');
        }
    },


    async deleteRow(idModulo) {
        const idPerfil = document.getElementById('selectPerfil').value;
        if (!idPerfil) return;

        // 1. Confirmación antes de borrar usando SweetAlert2
        let confirmar = true;
        if (window.Swal) {
            const result = await Swal.fire({
                title: '¿Limpiar permisos?',
                text: "Se quitarán todos los permisos de este módulo para el perfil seleccionado.",
                icon: 'warning',
                showCancelButton: true,
                confirmButtonColor: '#dc3545',
                cancelButtonColor: '#6c757d',
                confirmButtonText: 'Sí, limpiar',
                cancelButtonText: 'Cancelar'
            });
            confirmar = result.isConfirmed;
        } else {
            confirmar = confirm("¿Quitar todos los permisos de este módulo?");
        }

        if (!confirmar) return;

        // 2. Buscar el módulo en la memoria local
        const item = this.currentPermisos.find(p => p.idModulo === idModulo);
        if (!item) return;

        // Respaldar el estado actual por si el servidor falla
        const backupItem = { ...item };

        // 3. Poner todos los permisos en false
        item.bitAgregar = false;
        item.bitEditar = false;
        item.bitEliminar = false;
        item.bitConsulta = false;
        item.bitDetalle = false;
        item.bitImprimir = false;
        item.bitBitacora = false;

        // Actualizar la tabla visualmente de inmediato
        this.renderTable(true);

        // 4. Enviar los cambios al backend (reutilizando la lógica de autoSave)
        try {
            const res = await fetch('/api/permisos_perfil', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    id_perfil: parseInt(idPerfil), // ✅ CORREGIDO: Ajustado al DTO de Rust
                    permisos: [item] 
                })
            });

            const result = await res.json();
            if (result.success) {
                this.showToast('Permisos limpiados correctamente', 'success');
            } else {
                throw new Error(result.msg || "Error desconocido");
            }
        } catch (e) {
            // Si hay error en la BD, revertir los checkboxes a como estaban antes
            Object.assign(item, backupItem);
            this.renderTable(true);
            this.showToast("Error al limpiar permisos: " + e.message, 'error');
        }
    },

    // Sistema de notificaciones rescatado (Toaster)
    showToast(msg, type = 'success', timer = 2500) {
        if (window.Swal) {
            Swal.fire({
                toast: true,
                position: 'top-end',
                icon: type, 
                title: msg,
                showConfirmButton: false,
                timer: timer,
                timerProgressBar: true
            });
        } else {
            alert(`${type.toUpperCase()}: ${msg}`);
        }
    }
};

document.addEventListener('DOMContentLoaded', () => PermisosManager.init());