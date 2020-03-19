package dev.gridio.dnp3.codegen.model.groups

import dev.gridio.dnp3.codegen.model.FixedSizeField._
import dev.gridio.dnp3.codegen.model.VariationNames._
import dev.gridio.dnp3.codegen.model.{AnyVariation, FixedSize, GroupVariation, ObjectGroup}

object Group30 extends ObjectGroup {
  def variations: List[GroupVariation] = List(Group30Var0, Group30Var1, Group30Var2, Group30Var3, Group30Var4, Group30Var5, Group30Var6)

  def group: Byte = 30

  def desc: String = "Analog Input"

  def isEventGroup: Boolean = false
}

object Group30Var0 extends AnyVariation(Group30, 0)

object Group30Var1 extends FixedSize(Group30, 1, bit32WithFlag)(flags, value32)

object Group30Var2 extends FixedSize(Group30, 2, bit16WithFlag)(flags, value16)

object Group30Var3 extends FixedSize(Group30, 3, bit32WithoutFlag)(value32)

object Group30Var4 extends FixedSize(Group30, 4, bit16WithoutFlag)(value16)

object Group30Var5 extends FixedSize(Group30, 5, singlePrecisionWithFlag)(flags, float32)

object Group30Var6 extends FixedSize(Group30, 6, doublePrecisionWithFlag)(flags, float64)
