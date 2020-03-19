package dev.gridio.dnp3.codegen.model.groups

import dev.gridio.dnp3.codegen.model.FixedSizeField._
import dev.gridio.dnp3.codegen.model.VariationNames._
import dev.gridio.dnp3.codegen.model.{AnyVariation, FixedSize, GroupVariation, ObjectGroup}

object Group41 extends ObjectGroup {
  def variations: List[GroupVariation] = List(Group41Var0, Group41Var1, Group41Var2, Group41Var3, Group41Var4)

  def group: Byte = 41

  def desc: String = "Analog Output"

  def isEventGroup: Boolean = false
}

object Group41Var0 extends AnyVariation(Group41, 0)

object Group41Var1 extends FixedSize(Group41, 1, bit32WithFlag)(value32, commandStatus)

object Group41Var2 extends FixedSize(Group41, 2, bit16WithFlag)(value16, commandStatus)

object Group41Var3 extends FixedSize(Group41, 3, singlePrecision)(float32, commandStatus)

object Group41Var4 extends FixedSize(Group41, 4, doublePrecision)(float64, commandStatus)