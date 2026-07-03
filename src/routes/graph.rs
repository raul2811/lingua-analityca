use std::collections::{HashMap, HashSet};

use crate::models::{
    AristaGrafoDependencia, Dependencia, GrafoDependencia, LineaArbolDependencia,
    NodoGrafoDependencia,
};

use super::tokens::map_linguakit_to_ud;

pub(super) fn construir_arbol(deps: &[Dependencia]) -> Vec<LineaArbolDependencia> {
    if deps.is_empty() {
        return vec![LineaArbolDependencia {
            indent_px: 0,
            texto: "Sin dependencias disponibles".to_string(),
        }];
    }

    let mut hijos: HashMap<String, Vec<&Dependencia>> = HashMap::new();
    let mut gobernadores = HashSet::new();
    let mut dependientes = HashSet::new();

    for dep in deps {
        gobernadores.insert(dep.gobernador.clone());
        dependientes.insert(dep.dependiente.clone());
        hijos.entry(dep.gobernador.clone()).or_default().push(dep);
    }

    let mut raices: Vec<String> = gobernadores.difference(&dependientes).cloned().collect();
    raices.sort();

    if raices.is_empty() {
        raices = gobernadores.into_iter().collect();
        raices.sort();
    }

    let mut lineas = Vec::new();
    let mut visitados = HashSet::new();

    for raiz in raices.into_iter().take(3) {
        recorrer_arbol(&raiz, &raiz, 0, &hijos, &mut visitados, &mut lineas);
    }

    lineas
}

pub(super) fn construir_grafo_dependencias(deps: &[Dependencia]) -> GrafoDependencia {
    if deps.is_empty() {
        return GrafoDependencia {
            width: 360,
            height: 140,
            nodos: vec![NodoGrafoDependencia {
                id: "sin-dependencias".to_string(),
                texto: "Sin dependencias disponibles".to_string(),
                x: 40,
                y: 50,
            }],
            aristas: vec![],
        };
    }

    let mut hijos: HashMap<String, Vec<&Dependencia>> = HashMap::new();
    let mut gobernadores = HashSet::new();
    let mut dependientes = HashSet::new();

    for dep in deps {
        gobernadores.insert(dep.gobernador.clone());
        dependientes.insert(dep.dependiente.clone());
        hijos.entry(dep.gobernador.clone()).or_default().push(dep);
    }

    let mut raices: Vec<String> = gobernadores.difference(&dependientes).cloned().collect();
    raices.sort();

    if raices.is_empty() {
        raices = gobernadores.into_iter().collect();
        raices.sort();
    }

    let mut posiciones: HashMap<String, (usize, usize)> = HashMap::new();
    let mut orden = 0;
    let mut visitados = HashSet::new();

    for raiz in raices {
        asignar_posiciones_grafo(
            &raiz,
            0,
            &hijos,
            &mut visitados,
            &mut posiciones,
            &mut orden,
        );
    }

    let mut nodos: Vec<NodoGrafoDependencia> = posiciones
        .iter()
        .map(|(texto, (x, y))| NodoGrafoDependencia {
            id: id_nodo(texto),
            texto: texto.clone(),
            x: *x,
            y: *y,
        })
        .collect();
    nodos.sort_by_key(|n| (n.y, n.x));

    let aristas: Vec<AristaGrafoDependencia> = deps
        .iter()
        .filter_map(|dep| {
            let (x1, y1) = *posiciones.get(&dep.gobernador)?;
            let (x2, y2) = *posiciones.get(&dep.dependiente)?;

            Some(AristaGrafoDependencia {
                desde: id_nodo(&dep.gobernador),
                hacia: id_nodo(&dep.dependiente),
                relacion: dep.relacion.clone(),
                x1: x1 + 70,
                y1: y1 + 34,
                x2: x2 + 70,
                y2,
                label_x: (x1 + x2) / 2 + 48,
                label_y: (y1 + y2) / 2 + 8,
                ud_aproximado: map_linguakit_to_ud(&dep.relacion).to_string(),
                etiqueta_visual: etiqueta_arista(&dep.relacion),
            })
        })
        .collect();

    let width = nodos.iter().map(|n| n.x).max().unwrap_or(360) + 190;
    let height = nodos.iter().map(|n| n.y).max().unwrap_or(140) + 100;

    GrafoDependencia {
        width,
        height,
        nodos,
        aristas,
    }
}

fn asignar_posiciones_grafo(
    nodo: &str,
    profundidad: usize,
    hijos: &HashMap<String, Vec<&Dependencia>>,
    visitados: &mut HashSet<String>,
    posiciones: &mut HashMap<String, (usize, usize)>,
    orden: &mut usize,
) {
    if !visitados.insert(nodo.to_string()) {
        return;
    }

    let x = 40 + (*orden * 165);
    let y = 40 + (profundidad * 92);
    posiciones.insert(nodo.to_string(), (x, y));
    *orden += 1;

    if let Some(deps) = hijos.get(nodo) {
        for dep in deps {
            asignar_posiciones_grafo(
                &dep.dependiente,
                profundidad + 1,
                hijos,
                visitados,
                posiciones,
                orden,
            );
        }
    }
}

fn id_nodo(texto: &str) -> String {
    texto
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect()
}

fn etiqueta_arista(relacion: &str) -> String {
    let ud = map_linguakit_to_ud(relacion);

    if ud != "dep" && ud != relacion {
        format!("{relacion} ≈ {ud}")
    } else {
        relacion.to_string()
    }
}

fn recorrer_arbol(
    nodo: &str,
    texto: &str,
    nivel: usize,
    hijos: &HashMap<String, Vec<&Dependencia>>,
    visitados: &mut HashSet<String>,
    lineas: &mut Vec<LineaArbolDependencia>,
) {
    if !visitados.insert(nodo.to_string()) {
        return;
    }

    lineas.push(LineaArbolDependencia {
        indent_px: nivel * 20,
        texto: texto.to_string(),
    });

    if let Some(deps) = hijos.get(nodo) {
        for dep in deps {
            let texto = format!("{} [{}]", dep.dependiente, dep.relacion);
            recorrer_arbol(
                &dep.dependiente,
                &texto,
                nivel + 1,
                hijos,
                visitados,
                lineas,
            );
        }
    }
}
